use std::collections::HashMap;
use std::ops::Index;

use crate::{ObjectId, Point, Say, Sayer, Arrow, Writable};

#[cfg(test)]
mod tests {
	use crate::{Object, ObjectId, Point, Arrow};

	const COUNT: Point = Point::Static { name: "count", aspect: "Counter" };

	#[test]
	fn index() {
		let object = Object::new(
			&ObjectId::String("MyCounter".into()),
			vec![(&COUNT, Some(Arrow::Number(17)))],
		);
		let count = &object[&COUNT];
		assert_eq!(count, &Arrow::Number(17))
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Object {
	pub id: ObjectId,
	pub properties: HashMap<Point, Arrow>,
}

impl Object {
	pub fn insert(&mut self, point: &Point, arrow: Arrow) {
		let mut properties = self.properties.clone();
		properties.insert(point.clone(), arrow);
		self.properties = properties
	}
	pub fn new(object_id: &ObjectId, properties: Vec<(&Point, Option<Arrow>)>) -> Self {
		let mut map = HashMap::new();
		for (point, arrow) in properties {
			if let Some(arrow) = arrow {
				map.insert(point.to_owned(), arrow);
			}
		}
		Object { id: object_id.to_owned(), properties: map }
	}
	pub fn new_with_id(object_id: &ObjectId) -> Self { Object { id: object_id.to_owned(), properties: HashMap::new() } }
}

impl Index<&Point> for Object {
	type Output = Arrow;
	fn index(&self, index: &Point) -> &Self::Output { &self.properties[index] }
}

impl Writable for Object {
	fn to_says(&self) -> Vec<Say> {
		self.properties.keys()
			.map(|point| Say {
				sayer: Sayer::Unit,
				object: self.id.to_owned(),
				point: point.to_owned(),
				arrow: self.properties.get(point).map(Arrow::to_owned),
			})
			.collect()
	}
}
