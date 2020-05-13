use std::{io, thread};
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

use crate::{AmpContext, AmpScope, Chamber, Object, Point, Say, Sayer, Speech, Target};
use crate::diary::Diary;
use crate::hamt::{Hamt, Root};
use crate::util::io_error;

pub use self::key::*;

mod key;

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
		let echo = Echo { tx };
		thread::spawn(move || {
			let diary = Diary::temp().unwrap();
			let mut diary_writer = diary.writer().unwrap();
			let mut hamt2 = Hamt::new(Root::ZERO);
			for action in rx {
				match action {
					Action::Speech(speech, tx) => {
						// TODO Deal with reader and write unwraps.
						for say in speech.says {
							let key = say.as_echo_key();
							hamt2.write_value(&key, &say.target, &mut diary_writer).unwrap();
						}
						diary.commit(diary_writer.end_size());
						let chamber = Chamber {
							reader: hamt2.reader().unwrap(),
							diary_reader: diary.reader().unwrap(),
						};
						tx.send(Ok(chamber)).unwrap();
					}
					Action::Latest(tx) => {
						// TODO Deal with reader unwrap.
						let chamber = Chamber {
							reader: hamt2.reader().unwrap(),
							diary_reader: diary.reader().unwrap(),
						};
						tx.send(chamber).unwrap();
					}
				}
			}
		});
		echo
	}
}

impl Say {
	pub(crate) fn as_echo_key(&self) -> EchoKey {
		EchoKey::SayerObjjectPoint(self.sayer.clone(), self.object.clone(), self.point.clone())
	}
}

