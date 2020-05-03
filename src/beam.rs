use crate::{Said, Say, Sayer, Ship, Speech, Subject};

pub(crate) struct BeamScope {
	pub(crate) says: Vec<Say>
}

impl BeamScope {
	pub fn stanza(self) -> Speech {
		Speech { says: self.says }
	}
}

pub trait BeamContext {
	fn say(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship, said: &Said);
}

impl BeamContext for BeamScope {
	fn say(&mut self, sayer: &Sayer, subject: &Subject, ship: &Ship, said: &Said) {
		let say = Say::Assert(sayer.clone(), subject.clone(), ship.clone(), said.clone());
		self.says.push(say);
	}
}
