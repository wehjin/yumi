use crate::{Target, Say, Sayer, Point, Speech, Subject};

pub(crate) struct AmpScope {
	pub(crate) says: Vec<Say>
}

impl AmpScope {
	pub fn speech(self) -> Speech {
		Speech { says: self.says }
	}
}

pub trait AmpContext {
	fn say(&mut self, sayer: &Sayer, subject: &Subject, point: &Point, target: &Target);
}

impl AmpContext for AmpScope {
	fn say(&mut self, sayer: &Sayer, subject: &Subject, point: &Point, target: &Target) {
		let say = Say {
			sayer: sayer.clone(),
			subject: subject.clone(),
			point: point.clone(),
			target: Some(target.clone()),
		};
		self.says.push(say);
	}
}
