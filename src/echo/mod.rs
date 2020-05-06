use std::{io, thread};
use std::error::Error;
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

use crate::{AmpContext, AmpScope, Chamber, Said, Say, Sayer, Ship, Speech, Subject};
use crate::hamt::Hamt;
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
	pub fn write(&self, said: Said) -> io::Result<Chamber> {
		let say = Say { sayer: Sayer::Unit, subject: Subject::Unit, ship: Ship::Unit, said: Some(said) };
		let speech = Speech { says: vec![say] };
		self.send_speech(speech)
	}

	pub fn batch_write(&self, f: impl FnOnce(&mut dyn AmpContext) -> ()) -> io::Result<Chamber> {
		let mut amp = AmpScope { says: Vec::new() };
		f(&mut amp);
		self.send_speech(amp.speech())
	}

	fn send_speech(&self, speech: Speech) -> io::Result<Chamber> {
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
			let mut hamt = Hamt::new();
			for action in rx {
				match action {
					Action::Speech(speech, tx) => {
						let init = hamt.extender();
						let extender = speech.says.iter().fold(
							Ok(init),
							|result, say| {
								if let Ok(extender) = result {
									let key = say.as_echo_key();
									let extension = extender.extend(&key, say);
									extension
								} else { result }
							},
						);
						let chamber = extender.map(|extender| {
							hamt.commit(extender);
							Chamber { origin: thread_echo.to_owned(), viewer: hamt.viewer() }
						});
						tx.send(chamber).unwrap();
					}
					Action::Latest(tx) => {
						let ray = Chamber {
							origin: thread_echo.to_owned(),
							viewer: hamt.viewer(),
						};
						tx.send(ray).unwrap();
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

