use crate::{diary, Echo, Sayer, Point, Subject, Target};
use crate::echo::EchoKey;
use crate::hamt::Reader;

pub struct Chamber {
	pub(crate) origin: Echo,
	pub(crate) reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}

impl Chamber {
	pub fn full_read(&mut self, sayer: &Sayer, subject: &Subject, point: &Point) -> Option<Target> {
		let key = EchoKey::SayerSubjectPoint(sayer.clone(), subject.clone(), point.clone());
		self.target(&key)
	}

	fn target(&mut self, key: &EchoKey) -> Option<Target> {
		let target: Option<Option<Target>> = self.reader.read_value(key, &mut self.diary_reader).unwrap();
		target.unwrap_or(None)
	}

	pub fn read(&mut self) -> Option<Target> {
		let key = EchoKey::SayerSubjectPoint(Sayer::Unit, Subject::Unit, Point::Main);
		self.target(&key)
	}

	pub fn origin(&self) -> &Echo { &self.origin }
}

