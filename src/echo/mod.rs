use std::{io, thread};
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

use crate::{AmpContext, AmpScope, Chamber, diary, Object, Point, Say, Sayer, Speech, Target};
use crate::diary::Diary;
use crate::hamt::{Hamt, Root};
use crate::util::io_error;

#[derive(Debug, Clone)]
pub struct Echo {
	tx: SyncSender<Action>
}

enum Action {
	Speech(Speech, Sender<io::Result<Chamber>>),
	Latest(Sender<Chamber>),
}

impl Echo {
	pub fn object_attributes(&mut self, object: &Object, attributes: Vec<(&Point, Target)>) -> io::Result<Chamber> {
		let says = attributes.into_iter().map(|(point, target)| {
			Say { sayer: Sayer::Unit, object: object.to_owned(), point: point.to_owned(), target: Some(target) }
		}).collect::<Vec<_>>();
		self.send_speech(Speech { says })?;
		self.chamber()
	}

	pub fn attributes(&mut self, attributes: Vec<(&Point, Target)>) -> io::Result<Chamber> {
		self.object_attributes(&Object::Unit, attributes)
	}

	pub fn target(&mut self, target: Target) -> io::Result<Chamber> {
		let say = Say { sayer: Sayer::Unit, object: Object::Unit, point: Point::Unit, target: Some(target) };
		let speech = Speech { says: vec![say] };
		self.send_speech(speech)
	}

	pub fn batch_write(&mut self, f: impl FnOnce(&mut dyn AmpContext) -> ()) -> io::Result<Chamber> {
		let mut amp = AmpScope { says: Vec::new() };
		f(&mut amp);
		self.send_speech(amp.speech())
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

	pub fn connect() -> Echo {
		let (tx, rx) = sync_channel::<Action>(64);
		thread::spawn(move || {
			let diary = Diary::temp().unwrap();
			let mut diary_writer = diary.writer().unwrap();
			let mut object_points = Hamt::new(Root::ZERO);
			for action in rx {
				match action {
					Action::Speech(speech, tx) => {
						let chamber: io::Result<Chamber> = speech.says.into_iter()
							.map(|say| write_say(&say, &mut object_points, &mut diary_writer))
							.collect::<io::Result<Vec<_>>>()
							.and_then(|_| {
								diary.commit(diary_writer.end_size());
								Ok(())
							})
							.and_then(|_| chamber(&object_points, &diary));
						tx.send(chamber).unwrap();
					}
					Action::Latest(tx) => {
						// TODO Deal with reader unwrap.
						let chamber = Chamber {
							reader: object_points.reader().unwrap(),
							diary_reader: diary.reader().unwrap(),
						};
						tx.send(chamber).unwrap();
					}
				}
			}
		});
		Echo { tx }
	}
}

fn chamber(object_points: &Hamt, diary: &Diary) -> io::Result<Chamber> {
	let reader = object_points.reader()?;
	let diary_reader = diary.reader()?;
	let chamber = Chamber { reader, diary_reader };
	Ok(chamber)
}

fn write_say(say: &Say, object_points: &mut Hamt, diary_writer: &mut diary::Writer) -> io::Result<()> {
	let mut diary_reader = diary_writer.reader()?;
	let points_root = match object_points.reader()?.read_value(&say.object, &mut diary_reader)? {
		None => Root::ZERO,
		Some(root) => root
	};
	let mut point_targets = Hamt::new(points_root);
	let target = match say.target {
		None => unimplemented!(),
		Some(it) => it,
	};
	point_targets.write_value(&say.point, &target, diary_writer)?;
	object_points.write_value(&say.object, &point_targets.root, diary_writer)?;
	Ok(())
}
