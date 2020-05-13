use std::io;

use crate::{diary, Object, Point, Target};
use crate::hamt::{Hamt, Reader};

pub struct Chamber {
	pub(crate) reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}

impl Chamber {
	pub fn object_attributes<'a>(&mut self, object: &'a Object, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		points.into_iter().map(|point| {
			let target = self.read_target(object, point).unwrap_or(None);
			(point, target)
		}).collect()
	}

	pub fn attributes<'a>(&mut self, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		self.object_attributes(&Object::Unit, points)
	}

	pub fn target(&mut self) -> Option<Target> {
		self.read_target(&Object::Unit, &Point::Unit).unwrap_or(None)
	}

	fn read_target(&mut self, object: &Object, point: &Point) -> io::Result<Option<Target>> {
		match self.reader.read_value(object, &mut self.diary_reader).unwrap_or(None) {
			None => Ok(None),
			Some(root) => {
				let point_targets = Hamt::new(root);
				point_targets.reader()?.read_value(point, &mut self.diary_reader)
			}
		}
	}
}
