use crate::{hamt, Nova, Said, Say, Ship, Subject};

pub struct Ray {
	pub(crate) origin: Nova,
	pub(crate) viewer: hamt::Viewer<Say>,
}

impl Ray {
	pub fn said(&self, subject: &Subject, _ship: &Ship) -> Option<&Said> {
		let say = self.viewer.value(subject);
		let said = say.map(Say::said).flatten();
		said
	}
	pub fn origin(&self) -> &Nova { &self.origin }
}