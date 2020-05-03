use crate::{Nova, Said, Ship, Subject};

pub struct Ray {
	pub(crate) origin: Nova
}

impl Ray {
	pub fn said(&self, subject: &Subject, ship: &Ship) -> Said {
		Said::Number(2435)
	}


	pub fn origin(&self) -> &Nova { &self.origin }
}
