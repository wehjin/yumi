use crate::{Melody, Song};

pub(crate) struct BeamScope {
	pub(crate) melodies: Vec<Melody>
}

impl BeamScope {
	pub fn stanza(self) -> Song {
		Song { melodies: self.melodies }
	}
}

pub trait BeamContext {
	fn add(&mut self, melody: Melody);
}

impl BeamContext for BeamScope {
	fn add(&mut self, melody: Melody) {
		self.melodies.push(melody)
	}
}
