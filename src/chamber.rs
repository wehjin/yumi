use std::io;

use crate::{diary, ObjectId, Point, Target};
use crate::hamt::{Hamt, ProdAB, Reader, Root};

pub struct Chamber {
	pub(crate) object_points_reader: Reader,
	pub(crate) point_objects_reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}


impl Chamber {
	pub fn objects<'a, F: ObjectFilter<'a>>(&mut self) -> io::Result<Vec<F>> {
		let obj_names = self.objects_with_point(F::key_point())?;
		let objects = obj_names.into_iter()
			.map(|obj_name| {
				let properties = self.object_properties(&obj_name, F::data_points().to_vec());
				F::from_name_and_properties(&obj_name, properties)
			}).collect::<Vec<_>>();
		Ok(objects)
	}

	pub fn objects_with_property(&self, point: &Point, target: &Target) -> io::Result<Vec<ObjectId>> {
		let mut matching_objects = Vec::new();
		for object in self.objects_with_point(point)? {
			if target.eq(&self.read_target(&object, point)?.unwrap()) {
				matching_objects.push(object)
			}
		}
		Ok(matching_objects)
	}

	pub fn objects_with_point(&self, point: &Point) -> io::Result<Vec<ObjectId>> {
		let mut diary_reader = self.diary_reader.clone();
		self.inner_objects_with_point(point, &mut diary_reader)
	}

	fn inner_objects_with_point(&self, point: &Point, reader: &mut diary::Reader) -> io::Result<Vec<ObjectId>> {
		let objects_root: Option<Root> = self.point_objects_reader.read_value(point, reader)?;
		let objects = match objects_root {
			None => Vec::new(),
			Some(root) => {
				let object_target_reader = Hamt::new(root).reader()?;
				let object_target = object_target_reader.read_all::<ProdAB<ObjectId, Target>>(reader)?;
				object_target.into_iter().map(|it| it.a).collect()
			}
		};
		Ok(objects)
	}

	pub fn object_properties<'a>(&mut self, object: &'a ObjectId, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		points.into_iter().map(|point| {
			let target = self.read_target(object, point).unwrap_or(None);
			(point, target)
		}).collect()
	}

	pub fn properties<'a>(&mut self, points: Vec<&'a Point>) -> Vec<(&'a Point, Option<Target>)> {
		self.object_properties(&ObjectId::Unit, points)
	}

	pub fn target(&mut self) -> Option<Target> {
		self.read_target(&ObjectId::Unit, &Point::Unit).unwrap_or(None)
	}

	pub fn string(&self, object: &ObjectId, point: &Point) -> String {
		self.object_point(object, point).as_str().to_string()
	}

	pub fn number(&self, object: &ObjectId, point: &Point) -> u64 {
		self.object_point(object, point).as_number()
	}

	pub fn object_id(&self, object: &ObjectId, point: &Point) -> ObjectId {
		self.object_point(object, point).as_object_id().to_owned()
	}

	pub fn object_point(&self, object: &ObjectId, point: &Point) -> Target {
		self.read_target(object, point).unwrap().unwrap()
	}

	fn read_target(&self, object: &ObjectId, point: &Point) -> io::Result<Option<Target>> {
		let mut reader = self.diary_reader.clone();
		let root: Option<Root> = self.object_points_reader.read_value(object, &mut reader)?;
		match root {
			None => Ok(None),
			Some(root) => {
				let point_targets = Hamt::new(root);
				point_targets.reader()?.read_value(point, &mut reader)
			}
		}
	}
}

pub trait ObjectFilter<'a> {
	fn key_point() -> &'a Point;
	fn data_points() -> &'a [&'a Point];
	fn from_name_and_properties(obj_name: &ObjectId, properties: Vec<(&Point, Option<Target>)>) -> Self;
}
