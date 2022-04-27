use crate::{ObjectId, Ring, Say, Sayer, Arrow, Writable};
use crate::util::unique_name;

/// WriteScope allows a function to write facts into the database.
pub struct WriteScope {
	pub says: Vec<Say>
}

impl WriteScope {
	pub fn new_object_id(&self, prefix: &str) -> ObjectId { ObjectId::String(unique_name(prefix)) }

	pub fn writable(&mut self, writable: &impl Writable) {
		self.says(writable.to_says())
	}

	pub fn write_object_properties(&mut self, object: &ObjectId, properties: Vec<(&Ring, Arrow)>) {
		for (ring, arrow) in properties {
			let say = Say { sayer: Sayer::Unit, object: object.to_owned(), ring: ring.to_owned(), arrow: Some(arrow) };
			self.says.push(say)
		}
	}

	pub fn attributes(&mut self, attributes: Vec<(&Ring, Arrow)>) {
		self.write_object_properties(&ObjectId::Unit, attributes)
	}

	pub fn arrow(&mut self, arrow: Arrow) {
		self.attributes(vec![(&Ring::Unit, arrow)])
	}

	fn says(&mut self, says: Vec<Say>) {
		self.says.extend(says);
	}
}
