use crate::{ObjectId, Point, Say, Sayer, Target, Writable};
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

	pub fn write_object_properties(&mut self, object: &ObjectId, properties: Vec<(&Point, Target)>) {
		for (point, target) in properties {
			let say = Say { sayer: Sayer::Unit, object: object.to_owned(), point: point.to_owned(), target: Some(target) };
			self.says.push(say)
		}
	}

	pub fn attributes(&mut self, attributes: Vec<(&Point, Target)>) {
		self.write_object_properties(&ObjectId::Unit, attributes)
	}

	pub fn target(&mut self, target: Target) {
		self.attributes(vec![(&Point::Unit, target)])
	}

	fn says(&mut self, says: Vec<Say>) {
		self.says.extend(says);
	}
}
