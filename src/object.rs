use std::collections::HashMap;
use std::ops::Index;

use crate::{ObjName, Point, Say, Sayer, Target, Writable};

#[cfg(test)]
mod tests {
	use crate::{Object, ObjName, Point, Target};

	const COUNT: Point = Point::Static { name: "count", aspect: "Counter" };

	#[test]
	fn index() {
		let object = Object::new(
			&ObjName::String("MyCounter".into()),
			vec![(&COUNT, Some(Target::Number(17)))],
		);
		let count = &object[&COUNT];
		assert_eq!(count, &Target::Number(17))
	}
}

#[derive(Debug, Eq, PartialEq)]
pub struct Object {
	pub name: ObjName,
	pub properties: HashMap<Point, Target>,
}

impl Object {
	pub fn new(obj_name: &ObjName, properties: Vec<(&Point, Option<Target>)>) -> Self {
		let mut map = HashMap::new();
		for (point, target) in properties {
			if let Some(target) = target {
				map.insert(point.to_owned(), target);
			}
		}
		Object { name: obj_name.to_owned(), properties: map }
	}
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
				object: self.name.to_owned(),
				point: point.to_owned(),
				target: self.properties.get(point).map(Target::to_owned),
			})
			.collect()
	}
}
