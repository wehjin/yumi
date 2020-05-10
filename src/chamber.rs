use crate::{diary, Echo, Sayer, Ship, Subject, T};
use crate::echo::EchoKey;
use crate::hamt::Reader;

pub struct Chamber {
	pub(crate) origin: Echo,
	pub(crate) reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}

impl Chamber {
	pub fn full_read(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship) -> Option<T> {
		let key = EchoKey::SayerSubjectShip(sayer.clone(), subject.clone(), ship.clone());
		self.target(&key)
	}

	fn target(&mut self, key: &EchoKey) -> Option<T> {
		let target: Option<Option<T>> = self.reader.read_value(key, &mut self.diary_reader).unwrap();
		target.unwrap_or(None)
	}

	pub fn read(&mut self) -> Option<T> {
		let key = EchoKey::SayerSubjectShip(Sayer::Unit, Subject::Unit, Ship::Unit);
		self.target(&key)
	}

	pub fn origin(&self) -> &Echo { &self.origin }
}

