use crate::{diary, Object, Point, Sayer, Target};
use crate::echo::EchoKey;
use crate::hamt::Reader;

pub struct Chamber {
	pub(crate) reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}

impl Chamber {
	pub fn full_read(&mut self, sayer: &Sayer, object: &Object, point: &Point) -> Option<Target> {
		let key = EchoKey::SayerObjjectPoint(sayer.clone(), object.clone(), point.clone());
		self.read(&key)
	}

	pub fn object_attributes<'a>(&mut self, object: &'a Object, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		points.into_iter().map(|point| {
			let key = EchoKey::SayerObjjectPoint(Sayer::Unit, object.to_owned(), point.to_owned());
			let target = self.read(&key);
			(point, target)
		}).collect()
	}

	pub fn attributes<'a>(&mut self, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		self.object_attributes(&Object::Unit, points)
	}

	pub fn target(&mut self) -> Option<Target> {
		let key = EchoKey::SayerObjjectPoint(Sayer::Unit, Object::Unit, Point::Unit);
		self.read(&key)
	}

	fn read(&mut self, key: &EchoKey) -> Option<Target> {
		let target: Option<Option<Target>> = self.reader.read_value(key, &mut self.diary_reader).unwrap();
		target.unwrap_or(None)
	}
}
