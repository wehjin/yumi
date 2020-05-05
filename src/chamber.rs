use crate::{Echo, hamt, Said, Say, Sayer, Ship, Subject};
use crate::echo::EchoKey;

pub struct Chamber {
	pub(crate) origin: Echo,
	pub(crate) viewer: hamt::Viewer<Say>,
}

impl Chamber {
	pub fn full_read(&self, sayer: &Sayer, subject: &Subject, ship: &Ship) -> Option<&Said> {
		let key = EchoKey::SayerSubjectShip(sayer.clone(), subject.clone(), ship.clone());
		self.said_of_key(&key)
	}

	fn said_of_key(&self, key: &EchoKey) -> Option<&Said> {
		let say = self.viewer.value(key);
		let said = say.map(Say::said).flatten();
		said
	}

	pub fn read(&self) -> Option<&Said> {
		let key = EchoKey::SayerSubjectShip(Sayer::None, Subject::None, Ship::None);
		self.said_of_key(&key)
	}

	pub fn origin(&self) -> &Echo { &self.origin }
}

