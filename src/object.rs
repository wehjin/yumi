use std::collections::HashMap;
use std::ops::Index;

use crate::{ObjectId, Ring, Say, Sayer, Arrow, Writable};

#[cfg(test)]
mod tests {
	use crate::{Object, ObjectId, Ring, Arrow};

	const COUNT: Ring = Ring::Static { name: "count", aspect: "Counter" };

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
	pub properties: HashMap<Ring, Arrow>,
}

impl Object {
	pub fn insert(&mut self, ring: &Ring, arrow: Arrow) {
		let mut properties = self.properties.clone();
		properties.insert(ring.clone(), arrow);
		self.properties = properties
	}
	pub fn new(object_id: &ObjectId, properties: Vec<(&Ring, Option<Arrow>)>) -> Self {
		let mut map = HashMap::new();
		for (ring, arrow) in properties {
			if let Some(arrow) = arrow {
				map.insert(ring.to_owned(), arrow);
			}
		}
		Object { id: object_id.to_owned(), properties: map }
	}
	pub fn new_with_id(object_id: &ObjectId) -> Self { Object { id: object_id.to_owned(), properties: HashMap::new() } }
}

impl Index<&Ring> for Object {
	type Output = Arrow;
	fn index(&self, index: &Ring) -> &Self::Output { &self.properties[index] }
}

impl Writable for Object {
	fn to_says(&self) -> Vec<Say> {
		self.properties.keys()
			.map(|ring| Say {
				sayer: Sayer::Unit,
				object: self.id.to_owned(),
				ring: ring.to_owned(),
				arrow: self.properties.get(ring).map(Arrow::to_owned),
			})
			.collect()
	}
}
