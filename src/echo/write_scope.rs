use crate::{ObjName, Point, Say, Sayer, Target, Writable};

pub struct WriteScope {
	pub says: Vec<Say>
}

impl WriteScope {
	pub fn writable(&mut self, writable: &impl Writable) {
		self.says(writable.to_says())
	}

	pub fn object_attributes(&mut self, object: &ObjName, attributes: Vec<(&Point, Target)>) {
		for (point, target) in attributes {
			let say = Say { sayer: Sayer::Unit, object: object.to_owned(), point: point.to_owned(), target: Some(target) };
			self.says.push(say)
		}
	}

	pub fn attributes(&mut self, attributes: Vec<(&Point, Target)>) {
		self.object_attributes(&ObjName::Unit, attributes)
	}

	pub fn target(&mut self, target: Target) {
		self.attributes(vec![(&Point::Unit, target)])
	}

	fn says(&mut self, mut says: Vec<Say>) {
		self.says.append(&mut says);
	}
}
