use crate::{T, Say, Sayer, Ship, Speech, Subject};

pub(crate) struct AmpScope {
	pub(crate) says: Vec<Say>
}

impl AmpScope {
	pub fn speech(self) -> Speech {
		Speech { says: self.says }
	}
}

pub trait AmpContext {
	fn say(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship, target: &T);
}

impl AmpContext for AmpScope {
	fn say(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship, target: &T) {
		let say = Say {
			sayer: sayer.clone(),
			subject: subject.clone(),
			ship: ship.clone(),
			target: Some(target.clone()),
		};
		self.says.push(say);
	}
}
