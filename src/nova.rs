use std::error::Error;
use std::io::Cursor;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

use crate::{BeamContext, BeamScope, hamt, Ray, Song};

#[derive(Debug, Clone)]
pub struct Nova {
	tx: SyncSender<NovaAction>
}

impl Nova {
	pub fn beam(&self, f: impl FnOnce(&mut dyn BeamContext) -> ()) -> Result<Ray, Box<dyn Error>> {
		let mut scope = BeamScope { melodies: Vec::new() };
		f(&mut scope);
		let stanza = scope.stanza();
		let (tx, rx) = sync_channel::<Ray>(1);
		let action = NovaAction::Sing(stanza, tx);
		self.tx.send(action).unwrap();
		let ray = rx.recv()?;
		Ok(ray)
	}

	pub fn latest(&self) -> Ray {
		Ray { origin: self.clone() }
	}

	pub fn connect() -> Nova {
		let (tx, rx) = sync_channel::<NovaAction>(64);
		let nova = Nova { tx };
		let thread_nova = nova.clone();
		thread::spawn(move || {
			let cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
			let mut writer = hamt::Writer::new(cursor, 0, 0);
			for action in rx {
				match action {
					NovaAction::Sing(_, tx) => {
						let ray = Ray { origin: thread_nova.clone() };
						tx.send(ray).unwrap();
					}
				}
			}
		});
		nova
	}
}

enum NovaAction {
	Sing(Song, SyncSender<Ray>)
}
