use std::collections::HashMap;
use std::ops::Index;

use crate::{ObjectId, Point, Say, Sayer, Target, Writable};

#[cfg(test)]
mod tests {
	use crate::{Object, ObjectId, Point, Target};

	const COUNT: Point = Point::Static { name: "count", aspect: "Counter" };

	#[test]
	fn index() {
		let object = Object::new(
			&ObjectId::String("MyCounter".into()),
			vec![(&COUNT, Some(Target::Number(17)))],
		);
		let count = &object[&COUNT];
		assert_eq!(count, &Target::Number(17))
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Object {
	pub id: ObjectId,
	pub properties: HashMap<Point, Target>,
}

impl Object {
	pub fn insert(&mut self, point: &Point, target: Target) {
		let mut properties = self.properties.clone();
		properties.insert(point.clone(), target);
		self.properties = properties
	}
	pub fn new(object_id: &ObjectId, properties: Vec<(&Point, Option<Target>)>) -> Self {
		let mut map = HashMap::new();
		for (point, target) in properties {
			if let Some(target) = target {
				map.insert(point.to_owned(), target);
			}
		}
		Object { id: object_id.to_owned(), properties: map }
	}
	pub fn new_with_id(object_id: &ObjectId) -> Self { Object { id: object_id.to_owned(), properties: HashMap::new() } }
}

impl Index<&Point> for Object {
	type Output = Target;
	fn index(&self, index: &Point) -> &Self::Output { &self.properties[index] }
}

impl Writable for Object {
	fn to_says(&self) -> Vec<Say> {
		self.properties.keys()
			.map(|point| Say {
				sayer: Sayer::Unit,
				object: self.id.to_owned(),
				point: point.to_owned(),
				target: self.properties.get(point).map(Target::to_owned),
			})
			.collect()
	}
}
