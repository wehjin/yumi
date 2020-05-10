use std::{io, thread};
use std::error::Error;
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

use crate::{AmpContext, AmpScope, Chamber, Say, Sayer, Ship, Speech, Subject, T};
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
	pub fn write(&mut self, target: T) -> io::Result<Chamber> {
		let say = Say { sayer: Sayer::Unit, subject: Subject::Unit, ship: Ship::Unit, target: Some(target) };
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

	pub fn latest(&self) -> Result<Chamber, Box<dyn Error>> {
		let (tx, rx) = channel::<Chamber>();
		let action = Action::Latest(tx);
		self.tx.send(action).unwrap();
		let ray = rx.recv()?;
		Ok(ray)
	}

	pub fn connect() -> Echo {
		let (tx, rx) = sync_channel::<Action>(64);
		let echo = Echo { tx };
		let thread_echo = echo.clone();
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
						diary.commit2(diary_writer.end_size());
						let chamber = Chamber {
							origin: thread_echo.to_owned(),
							reader: hamt2.reader().unwrap(),
							diary_reader: diary.reader().unwrap(),
						};
						tx.send(Ok(chamber)).unwrap();
					}
					Action::Latest(tx) => {
						// TODO Deal with reader unwrap.
						let chamber = Chamber {
							origin: thread_echo.to_owned(),
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
		EchoKey::SayerSubjectShip(self.sayer.clone(), self.subject.clone(), self.ship.clone())
	}
}

