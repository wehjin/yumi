use crate::Nova;

pub struct Ray {
	pub(crate) origin: Nova
}

impl Ray {
	pub fn origin(&self) -> &Nova { &self.origin }
}
