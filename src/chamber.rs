use crate::{Echo, hamt, Said, Sayer, Ship, Subject};
use crate::echo::EchoKey;

pub struct Chamber {
	pub(crate) origin: Echo,
	pub(crate) viewer: hamt::Viewer<Option<Said>>,
}

impl Chamber {
	pub fn full_read(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship) -> &Option<Said> {
		let key = EchoKey::SayerSubjectShip(sayer.clone(), subject.clone(), ship.clone());
		self.said_of_key(&key)
	}

	fn said_of_key(&mut self, key: &EchoKey) -> &Option<Said> {
		match self.viewer.value(key) {
			None => &None,
			Some(said) => said
		}
	}

	pub fn read(&mut self) -> &Option<Said> {
		let key = EchoKey::SayerSubjectShip(Sayer::Unit, Subject::Unit, Ship::Unit);
		self.said_of_key(&key)
	}

	pub fn origin(&self) -> &Echo { &self.origin }
}

