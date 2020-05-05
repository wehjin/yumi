use std::error::Error;
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};
use std::thread;

use crate::{BeamContext, BeamScope, Ray, Speech};
use crate::hamt::Hamt;

#[derive(Debug, Clone)]
pub struct Nova {
	tx: SyncSender<NovaAction>
}

enum NovaAction {
	Speak(Speech, Sender<Ray>),
	Latest(Sender<Ray>),
}

impl Nova {
	pub fn speak(&self, f: impl FnOnce(&mut dyn BeamContext) -> ()) -> Result<Ray, Box<dyn Error>> {
		let mut scope = BeamScope { says: Vec::new() };
		f(&mut scope);
		let stanza = scope.stanza();
		let (tx, rx) = channel::<Ray>();
		let action = NovaAction::Speak(stanza, tx);
		self.tx.send(action).unwrap();
		let ray = rx.recv()?;
		Ok(ray)
	}

	pub fn latest(&self) -> Result<Ray, Box<dyn Error>> {
		let (tx, rx) = channel::<Ray>();
		let action = NovaAction::Latest(tx);
		self.tx.send(action).unwrap();
		let ray = rx.recv()?;
		Ok(ray)
	}

	pub fn connect() -> Nova {
		let (tx, rx) = sync_channel::<NovaAction>(64);
		let nova = Nova { tx };
		let thread_nova = nova.clone();
		thread::spawn(move || {
			let mut hamt = Hamt::new();
			for action in rx {
				match action {
					NovaAction::Speak(speech, tx) => {
						let extender = speech.says.iter().fold(
							hamt.extender(),
							|extender, say| {
								let subject = say.subject();
								extender.extend(subject, say)
							},
						);
						hamt.commit(extender);
						let ray = Ray {
							origin: thread_nova.to_owned(),
							viewer: hamt.viewer(),
						};
						tx.send(ray).unwrap();
					}
					NovaAction::Latest(tx) => {
						let ray = Ray {
							origin: thread_nova.to_owned(),
							viewer: hamt.viewer(),
						};
						tx.send(ray).unwrap();
					}
				}
			}
		});
		nova
	}
}
