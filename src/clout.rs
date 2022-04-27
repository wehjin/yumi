use std::collections::HashMap;
use std::ops::Index;

use crate::{Arrow, Ring, Flight, Archer, Target, CanVolley};

#[cfg(test)]
mod tests {
	use crate::{Arrow, Clout, Ring, Target};

	const COUNT: Ring = Ring::Static { name: "count", aspect: "Counter" };

	#[test]
	fn index() {
		let clout = Clout::new(
			&Target::String("MyCounter".into()),
			vec![(&COUNT, Some(Arrow::Number(17)))],
		);
		let count = &clout[&COUNT];
		assert_eq!(count, &Arrow::Number(17))
	}
}

/// A `Clout` allows a user see what `Arrow`s are in a `Target` within a `Ring`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Clout {
	pub target: Target,
	pub properties: HashMap<Ring, Arrow>,
}

impl Clout {
	pub fn insert(&mut self, ring: &Ring, arrow: Arrow) {
		let mut properties = self.properties.clone();
		properties.insert(ring.clone(), arrow);
		self.properties = properties
	}
	pub fn new(target: &Target, properties: Vec<(&Ring, Option<Arrow>)>) -> Self {
		let mut map = HashMap::new();
		for (ring, arrow) in properties {
			if let Some(arrow) = arrow {
				map.insert(ring.to_owned(), arrow);
			}
		}
		Clout { target: target.to_owned(), properties: map }
	}
	pub fn new_with_target(target: &Target) -> Self { Clout { target: target.to_owned(), properties: HashMap::new() } }
}

impl Index<&Ring> for Clout {
	type Output = Arrow;
	fn index(&self, index: &Ring) -> &Self::Output { &self.properties[index] }
}

impl CanVolley for Clout {
	fn to_flights(&self) -> Vec<Flight> {
		self.properties.keys()
			.map(|ring| Flight {
				archer: Archer::Unit,
				target: self.target.to_owned(),
				ring: ring.to_owned(),
				arrow: self.properties.get(ring).map(Arrow::to_owned),
			})
			.collect()
	}
}
