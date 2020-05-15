use std::collections::HashSet;
use std::io;

use crate::{diary, ObjName, Point, Target};
use crate::hamt::{Hamt, ProdAB, Reader, Root};

pub struct Chamber {
	pub(crate) object_points_reader: Reader,
	pub(crate) point_objects_reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}

impl Chamber {
	pub fn objects_with_point(&mut self, point: &Point) -> io::Result<HashSet<ObjName>> {
		let objects_root: Option<Root> = self.point_objects_reader.read_value(point, &mut self.diary_reader)?;
		let objects = match objects_root {
			None => Vec::new(),
			Some(root) => {
				let object_target_reader = Hamt::new(root).reader()?;
				let object_target = object_target_reader.read_all::<ProdAB<ObjName, Target>>(&mut self.diary_reader)?;
				object_target.into_iter().map(|it| it.a).collect()
			}
		};
		Ok(objects.into_iter().collect())
	}

	pub fn object_attributes<'a>(&mut self, object: &'a ObjName, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		points.into_iter().map(|point| {
			let target = self.read_target(object, point).unwrap_or(None);
			(point, target)
		}).collect()
	}

	pub fn attributes<'a>(&mut self, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		self.object_attributes(&ObjName::Unit, points)
	}

	pub fn target(&mut self) -> Option<Target> {
		self.read_target(&ObjName::Unit, &Point::Unit).unwrap_or(None)
	}

	fn read_target(&mut self, object: &ObjName, point: &Point) -> io::Result<Option<Target>> {
		let root: Option<Root> = self.object_points_reader.read_value(object, &mut self.diary_reader)?;
		match root {
			None => Ok(None),
			Some(root) => {
				let point_targets = Hamt::new(root);
				point_targets.reader()?.read_value(point, &mut self.diary_reader)
			}
		}
	}
}
