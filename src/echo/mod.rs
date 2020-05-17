use std::{io, thread};
use std::path::Path;
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

use crate::{Chamber, diary, ObjName, Point, Say, Sayer, Speech, Target};
use crate::diary::Diary;
use crate::hamt::{Hamt, ProdAB, Root};
use crate::util::io_error;

#[derive(Debug, Clone)]
pub struct Echo {
	tx: SyncSender<Action>
}

enum Action {
	Speech(Speech, Sender<io::Result<Chamber>>),
	Latest(Sender<Chamber>),
}

pub trait Shout {
	fn object_attributes(&mut self, object: &ObjName, attributes: Vec<(&Point, Target)>);

	fn attributes(&mut self, attributes: Vec<(&Point, Target)>) {
		self.object_attributes(&ObjName::Unit, attributes)
	}
	fn target(&mut self, target: Target) {
		self.attributes(vec![(&Point::Unit, target)])
	}
}

struct EchoShout {
	says: Vec<Say>
}

impl Shout for EchoShout {
	fn object_attributes(&mut self, object: &ObjName, attributes: Vec<(&Point, Target)>) {
		for (point, target) in attributes {
			let say = Say { sayer: Sayer::Unit, object: object.to_owned(), point: point.to_owned(), target: Some(target) };
			self.says.push(say)
		}
	}
}

impl Echo {
	pub fn shout(&mut self, f: impl Fn(&mut dyn Shout)) -> io::Result<()> {
		let mut shout = EchoShout { says: Vec::new() };
		f(&mut shout);
		self.send_speech(Speech { says: shout.says })?;
		Ok(())
	}

	fn send_speech(&mut self, speech: Speech) -> io::Result<Chamber> {
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
		let diary_path = {
			let mut path = folder_path.to_path_buf();
			path.push("diary.dat");
			path
		};
		let (tx, rx) = sync_channel::<Action>(64);
		thread::spawn(move || {
			let diary = Diary::load(&diary_path).unwrap();
			let mut diary_writer = diary.writer().unwrap();
			let mut object_points = Hamt::new(Root::ZERO);
			let mut point_objects = Hamt::new(Root::ZERO);
			for action in rx {
				match action {
					Action::Speech(speech, tx) => {
						let chamber: io::Result<Chamber> = speech.says.into_iter()
							.map(|say| write_say(&say, &mut point_objects, &mut object_points, &mut diary_writer))
							.collect::<io::Result<Vec<_>>>()
							.and_then(|_| Ok(diary.commit(diary_writer.end_size())))
							.and_then(|_| chamber(&point_objects, &object_points, &diary));
						tx.send(chamber).unwrap();
					}
					Action::Latest(tx) => {
						// TODO Deal with reader unwrap.
						let chamber = chamber(&point_objects, &object_points, &diary).unwrap();
						tx.send(chamber).unwrap();
					}
				}
			}
		});
		Echo { tx }
	}
}

fn chamber(point_objects: &Hamt, object_points: &Hamt, diary: &Diary) -> io::Result<Chamber> {
	let object_points_reader = object_points.reader()?;
	let point_objects_reader = point_objects.reader()?;
	let diary_reader = diary.reader()?;
	let chamber = Chamber { point_objects_reader, object_points_reader, diary_reader };
	Ok(chamber)
}

fn write_say(say: &Say, point_objects: &mut Hamt, object_points: &mut Hamt, diary_writer: &mut diary::Writer) -> io::Result<()> {
	let mut diary_reader = diary_writer.reader()?;
	write_object_points(&say, object_points, diary_writer, &mut diary_reader)?;
	write_point_objects(&say, point_objects, diary_writer, &mut diary_reader)?;
	Ok(())
}

fn write_point_objects(say: &Say, point_objects: &mut Hamt, diary_writer: &mut diary::Writer, mut diary_reader: &mut diary::Reader) -> io::Result<()> {
	let object_targets_root = match point_objects.reader()?.read_value(&say.point, &mut diary_reader)? {
		None => Root::ZERO,
		Some(it) => it
	};
	let mut object_targets = Hamt::new(object_targets_root);
	let target = match say.target {
		None => unimplemented!(),
		Some(it) => it,
	};
	let object_target = ProdAB { a: say.object.to_owned(), b: target };
	object_targets.write_value(&say.object, &object_target, diary_writer)?;
	point_objects.write_value(&say.point, &object_targets.root, diary_writer)
}

fn write_object_points(say: &Say, object_points: &mut Hamt, diary_writer: &mut diary::Writer, mut diary_reader: &mut diary::Reader) -> io::Result<()> {
	let point_targets_root = match object_points.reader()?.read_value(&say.object, &mut diary_reader)? {
		None => Root::ZERO,
		Some(it) => it,
	};
	let mut point_targets = Hamt::new(point_targets_root);
	let target = match say.target {
		None => unimplemented!(),
		Some(it) => it,
	};
	point_targets.write_value(&say.point, &target, diary_writer)?;
	object_points.write_value(&say.object, &point_targets.root, diary_writer)
}
