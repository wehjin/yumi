use crate::{Said, Say, Sayer, Ship, Speech, Subject};

pub(crate) struct AmpScope {
	pub(crate) says: Vec<Say>
}

impl AmpScope {
	pub fn speech(self) -> Speech {
		Speech { says: self.says }
	}
}

pub trait AmpContext {
	fn say(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship, said: &Said);
}

impl AmpContext for AmpScope {
	fn say(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship, said: &Said) {
		let say = Say::Assert(sayer.clone(), subject.clone(), ship.clone(), said.clone());
		self.says.push(say);
	}
}
