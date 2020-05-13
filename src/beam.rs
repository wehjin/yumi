use crate::{Target, Say, Sayer, Point, Speech, Object};

pub(crate) struct AmpScope {
	pub(crate) says: Vec<Say>
}

impl AmpScope {
	pub fn speech(self) -> Speech {
		Speech { says: self.says }
	}
}

pub trait AmpContext {
	fn say(&mut self, sayer: &Sayer, object: &Object, point: &Point, target: &Target);
}

impl AmpContext for AmpScope {
	fn say(&mut self, sayer: &Sayer, object: &Object, point: &Point, target: &Target) {
		let say = Say {
			sayer: sayer.clone(),
			object: object.clone(),
			point: point.clone(),
			target: Some(target.clone()),
		};
		self.says.push(say);
	}
}
