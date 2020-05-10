use crate::{diary, Echo, Said, Sayer, Ship, Subject};
use crate::echo::EchoKey;
use crate::hamt::Reader;

pub struct Chamber {
	pub(crate) origin: Echo,
	pub(crate) reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}

impl Chamber {
	pub fn full_read(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship) -> Option<Said> {
		let key = EchoKey::SayerSubjectShip(sayer.clone(), subject.clone(), ship.clone());
		self.said_of_key(&key)
	}

	fn said_of_key(&mut self, key: &EchoKey) -> Option<Said> {
		let said: Option<Option<Said>> = self.reader.read_value(key, &mut self.diary_reader).unwrap();
		said.unwrap_or(None)
	}

	pub fn read(&mut self) -> Option<Said> {
		let key = EchoKey::SayerSubjectShip(Sayer::Unit, Subject::Unit, Ship::Unit);
		self.said_of_key(&key)
	}

	pub fn origin(&self) -> &Echo { &self.origin }
}

