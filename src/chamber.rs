use std::io;

use crate::{diary, ObjName, Point, Target};
use crate::hamt::{Hamt, ProdAB, Reader, Root};

pub struct Chamber {
	pub(crate) object_points_reader: Reader,
	pub(crate) point_objects_reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}


impl Chamber {
	pub fn filter<'a, F: Filter<'a>>(&mut self) -> io::Result<Vec<F>> {
		let obj_names = self.objects_with_point(F::key_point())?;
		let holders = obj_names.into_iter()
			.map(|obj_name| {
				let properties = self.object_properties(&obj_name, F::data_points().to_vec());
				F::from_name_and_properties(&obj_name, properties)
			}).collect::<Vec<_>>();
		Ok(holders)
	}

	pub fn objects_with_point(&mut self, point: &Point) -> io::Result<Vec<ObjName>> {
		let objects_root: Option<Root> = self.point_objects_reader.read_value(point, &mut self.diary_reader)?;
		let objects = match objects_root {
			None => Vec::new(),
			Some(root) => {
				let object_target_reader = Hamt::new(root).reader()?;
				let object_target = object_target_reader.read_all::<ProdAB<ObjName, Target>>(&mut self.diary_reader)?;
				object_target.into_iter().map(|it| it.a).collect()
			}
		};
		Ok(objects)
	}

	pub fn object_properties<'a>(&mut self, object: &'a ObjName, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		points.into_iter().map(|point| {
			let target = self.read_target(object, point).unwrap_or(None);
			(point, target)
		}).collect()
	}

	pub fn properties<'a>(&mut self, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		self.object_properties(&ObjName::Unit, points)
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

pub trait Filter<'a> {
	fn key_point() -> &'a Point;
	fn data_points() -> &'a [&'a Point];
	fn from_name_and_properties(obj_name: &ObjName, attributes: Vec<(&Point, Option<Target>)>) -> Self;
}
