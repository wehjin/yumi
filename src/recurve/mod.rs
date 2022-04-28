use std::{io, thread};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

pub use write_scope::DrawScope;

use crate::{Bundle, diary, Flight, hamt, Volley};
use crate::bytes::{ReadBytes, WriteBytes};
use crate::diary::Diary;
use crate::hamt::{Hamt, ProdAB, Root, ROOT_LEN};
use crate::util::io_error;

mod write_scope;

/// A `Recurve` is the entry point for releasing data into the database.
#[derive(Debug, Clone)]
pub struct Recurve {
	tx: SyncSender<Action>,
}

enum Action {
	Release(Volley, Sender<io::Result<Bundle>>),
	Latest(Sender<Bundle>),
}

impl Recurve {
	/// Connects to a Recurve instance.
	pub fn connect(name: &str, folder: impl AsRef<Path>) -> Self {
		let folder = folder.as_ref();
		let mut folder_path = folder.to_path_buf();
		folder_path.push(name);
		std::fs::create_dir_all(&folder_path).unwrap();
		let (tx, rx) = sync_channel::<Action>(64);
		thread::spawn(move || {
			let mut recurve = InnerRecurve::new(folder_path);
			for action in rx {
				match action {
					Action::Release(volley, tx) => {
						let new_bundle = recurve.release(volley);
						tx.send(new_bundle).unwrap();
					}
					Action::Latest(tx) => {
						let bundle = recurve.to_bundle().unwrap();
						tx.send(bundle).unwrap();
					}
				}
			}
		});
		Recurve { tx }
	}

	/// Opens a scope for writing facts to the database and provides it to the
	/// given function.
	pub fn draw<R>(&self, f: impl Fn(&mut DrawScope) -> R) -> io::Result<R> {
		let mut draw = DrawScope { flights: Vec::new() };
		let result = f(&mut draw);
		self.release(Volley { flights: draw.flights })?;
		Ok(result)
	}

	fn release(&self, volley: Volley) -> io::Result<Bundle> {
		let (tx, rx) = channel::<io::Result<Bundle>>();
		let action = Action::Release(volley, tx);
		self.tx.send(action).unwrap();
		rx.recv().map_err(io_error)?
	}

	/// Constructs a `Bundle` for reading facts from the database.
	pub fn to_bundle(&self) -> io::Result<Bundle> {
		let (tx, rx) = channel::<Bundle>();
		let action = Action::Latest(tx);
		self.tx.send(action).unwrap();
		rx.recv().map_err(io_error)
	}
}

struct InnerRecurve {
	diary: Diary,
	diary_writer: diary::Writer,
	target_rings: Hamt,
	ring_targets: Hamt,
	roots_log: RootsLog,
}

impl InnerRecurve {
	fn release(&mut self, volley: Volley) -> io::Result<Bundle> {
		for flight in volley.flights.into_iter() {
			let mut diary_reader = self.diary_writer.reader()?;
			self.write_target_rings(&flight, &mut diary_reader)?;
			self.write_ring_targets(&flight, &mut diary_reader)?;
		}
		self.diary.commit(self.diary_writer.end_size());
		self.roots_log.write_roots(self.target_rings.root, self.ring_targets.root)?;
		self.to_bundle()
	}

	fn write_ring_targets(&mut self, flight: &Flight, diary_reader: &mut diary::Reader) -> io::Result<()> {
		let target_arrows_root = match self.ring_targets.reader()?.read_value(&flight.ring, diary_reader)? {
			None => Root::ZERO,
			Some(root) => root
		};
		let mut target_arrows = Hamt::new(target_arrows_root);
		let arrow = match &flight.arrow {
			None => unimplemented!(),
			Some(it) => it.clone(),
		};
		let target_arrow = ProdAB { a: flight.target.to_owned(), b: arrow };
		target_arrows.write_value(&flight.target, &target_arrow, &mut self.diary_writer)?;
		self.ring_targets.write_value(&flight.ring, &target_arrows.root, &mut self.diary_writer)
	}

	fn write_target_rings(&mut self, flight: &Flight, diary_reader: &mut diary::Reader) -> io::Result<()> {
		let ring_arrows_root = match self.target_rings.reader()?.read_value(&flight.target, diary_reader)? {
			None => Root::ZERO,
			Some(it) => it,
		};
		let mut ring_arrows = Hamt::new(ring_arrows_root);
		let arrow = match &flight.arrow {
			None => unimplemented!(),
			Some(it) => it.clone(),
		};
		ring_arrows.write_value(&flight.ring, &arrow, &mut self.diary_writer)?;
		self.target_rings.write_value(&flight.target, &ring_arrows.root, &mut self.diary_writer)
	}

	fn to_bundle(&self) -> io::Result<Bundle> {
		let bundle = Bundle {
			ring_targets_reader: self.ring_targets.reader()?,
			target_rings_reader: self.target_rings.reader()?,
			diary_reader: self.diary.reader()?,
		};
		Ok(bundle)
	}

	fn new(folder_path: PathBuf) -> Self {
		let diary = Diary::load(&file_path("diary.dat", &folder_path)).unwrap();
		let diary_writer = diary.writer().unwrap();
		let roots_log = RootsLog::new(&folder_path).unwrap();
		let (target_rings_root, ring_targets_root) = roots_log.roots;
		let target_rings = Hamt::new(target_rings_root);
		let ring_targets = Hamt::new(ring_targets_root);
		InnerRecurve { diary, diary_writer, target_rings, ring_targets, roots_log }
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
				reader.seek(SeekFrom::End(-2 * hamt::ROOT_LEN as i64))?;
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
