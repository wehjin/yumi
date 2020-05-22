use std::{io, thread};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

use crate::{Chamber, diary, hamt, ObjName, Point, Say, Sayer, Speech, Target};
use crate::bytes::{ReadBytes, WriteBytes};
use crate::diary::Diary;
use crate::hamt::{Hamt, ProdAB, Root, ROOT_LEN};
use crate::util::io_error;

#[derive(Debug, Clone)]
pub struct Echo {
	tx: SyncSender<Action>
}

enum Action {
	Speech(Speech, Sender<io::Result<Chamber>>),
	Latest(Sender<Chamber>),
}

pub trait WriteScope {
	fn object_attributes(&mut self, object: &ObjName, attributes: Vec<(&Point, Target)>);

	fn attributes(&mut self, attributes: Vec<(&Point, Target)>) {
		self.object_attributes(&ObjName::Unit, attributes)
	}
	fn target(&mut self, target: Target) {
		self.attributes(vec![(&Point::Unit, target)])
	}
}

struct WriteContext {
	says: Vec<Say>
}

impl WriteScope for WriteContext {
	fn object_attributes(&mut self, object: &ObjName, attributes: Vec<(&Point, Target)>) {
		for (point, target) in attributes {
			let say = Say { sayer: Sayer::Unit, object: object.to_owned(), point: point.to_owned(), target: Some(target) };
			self.says.push(say)
		}
	}
}

impl Echo {
	pub fn write(&self, f: impl Fn(&mut dyn WriteScope)) -> io::Result<()> {
		let mut write = WriteContext { says: Vec::new() };
		f(&mut write);
		self.write_speech(Speech { says: write.says })?;
		Ok(())
	}

	fn write_speech(&self, speech: Speech) -> io::Result<Chamber> {
		let (tx, rx) = channel::<io::Result<Chamber>>();
		let action = Action::Speech(speech, tx);
		self.tx.send(action).unwrap();
		rx.recv().map_err(io_error)?
	}

	pub fn chamber(&self) -> io::Result<Chamber> {
		let (tx, rx) = channel::<Chamber>();
		let action = Action::Latest(tx);
		self.tx.send(action).unwrap();
		rx.recv().map_err(io_error)
	}

	pub fn connect(folder_path: &Path) -> Self {
		let folder_path = folder_path.to_path_buf();
		let (tx, rx) = sync_channel::<Action>(64);
		thread::spawn(move || {
			let mut echo = InnerEcho::new(folder_path);
			for action in rx {
				match action {
					Action::Speech(speech, tx) => {
						let new_chamber = echo.write_speech(speech);
						tx.send(new_chamber).unwrap();
					}
					Action::Latest(tx) => {
						let chamber = echo.chamber().unwrap();
						tx.send(chamber).unwrap();
					}
				}
			}
		});
		Echo { tx }
	}
}

struct InnerEcho {
	diary: Diary,
	diary_writer: diary::Writer,
	object_points: Hamt,
	point_objects: Hamt,
	roots_log: RootsLog,
}

impl InnerEcho {
	fn write_speech(&mut self, speech: Speech) -> io::Result<Chamber> {
		for say in speech.says.into_iter() {
			let mut diary_reader = self.diary_writer.reader()?;
			self.write_object_points(&say, &mut diary_reader)?;
			self.write_point_objects(&say, &mut diary_reader)?;
		}
		self.diary.commit(self.diary_writer.end_size());
		self.roots_log.write_roots(self.object_points.root, self.point_objects.root)?;
		self.chamber()
	}

	fn write_point_objects(&mut self, say: &Say, diary_reader: &mut diary::Reader) -> io::Result<()> {
		let object_targets_root = match self.point_objects.reader()?.read_value(&say.point, diary_reader)? {
			None => Root::ZERO,
			Some(root) => root
		};
		let mut object_targets = Hamt::new(object_targets_root);
		let target = match &say.target {
			None => unimplemented!(),
			Some(it) => it.clone(),
		};
		let object_target = ProdAB { a: say.object.to_owned(), b: target };
		object_targets.write_value(&say.object, &object_target, &mut self.diary_writer)?;
		self.point_objects.write_value(&say.point, &object_targets.root, &mut self.diary_writer)
	}

	fn write_object_points(&mut self, say: &Say, diary_reader: &mut diary::Reader) -> io::Result<()> {
		let point_targets_root = match self.object_points.reader()?.read_value(&say.object, diary_reader)? {
			None => Root::ZERO,
			Some(it) => it,
		};
		let mut point_targets = Hamt::new(point_targets_root);
		let target = match &say.target {
			None => unimplemented!(),
			Some(it) => it.clone(),
		};
		point_targets.write_value(&say.point, &target, &mut self.diary_writer)?;
		self.object_points.write_value(&say.object, &point_targets.root, &mut self.diary_writer)
	}

	fn chamber(&self) -> io::Result<Chamber> {
		let chamber = Chamber {
			point_objects_reader: self.point_objects.reader()?,
			object_points_reader: self.object_points.reader()?,
			diary_reader: self.diary.reader()?,
		};
		Ok(chamber)
	}

	fn new(folder_path: PathBuf) -> Self {
		let diary = Diary::load(&file_path("diary.dat", &folder_path)).unwrap();
		let diary_writer = diary.writer().unwrap();
		let roots_log = RootsLog::new(&folder_path).unwrap();
		let (object_points_root, point_objects_root) = roots_log.roots;
		let object_points = Hamt::new(object_points_root);
		let point_objects = Hamt::new(point_objects_root);
		InnerEcho { diary, diary_writer, object_points, point_objects, roots_log }
	}
}

struct RootsLog {
	appender: File,
	roots: (Root, Root),
}

impl RootsLog {
	pub fn write_roots(&mut self, a: Root, b: Root) -> io::Result<()> {
		let pos = self.appender.seek(SeekFrom::Current(0))?;
		let result = a.write_bytes(&mut self.appender)
			.and_then(|len| {
				assert_eq!(len, ROOT_LEN);
				b.write_bytes(&mut self.appender)
			})
			.map(|len| {
				assert_eq!(len, ROOT_LEN);
				()
			});
		if result.is_err() {
			self.appender.set_len(pos).unwrap();
			self.appender.seek(SeekFrom::Start(pos)).unwrap();
		}
		result
	}
	pub fn new(folder_path: &Path) -> io::Result<Self> {
		let file_path = file_path("roots.dat", folder_path);
		let appender = OpenOptions::new().create(true).append(true).open(&file_path)?;
		let roots = {
			let file_len = std::fs::metadata(&file_path)?.len();
			if file_len == 0 {
				(Root::ZERO, Root::ZERO)
			} else {
				let mut reader = OpenOptions::new().read(true).open(&file_path)?;
				reader.seek(SeekFrom::End(-2 * hamt::root::ROOT_LEN as i64))?;
				let a_root = Root::read_bytes(&mut reader)?;
				let b_root = Root::read_bytes(&mut reader)?;
				(a_root, b_root)
			}
		};
		Ok(RootsLog { appender, roots })
	}
}

fn file_path(file_name: &str, folder_path: &Path) -> PathBuf {
	let mut path = folder_path.to_path_buf();
	path.push(file_name);
	path
}
