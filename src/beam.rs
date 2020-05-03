use crate::{Say, Song};

pub(crate) struct BeamScope {
	pub(crate) melodies: Vec<Say>
}

impl BeamScope {
	pub fn stanza(self) -> Song {
		Song { melodies: self.melodies }
	}
}

pub trait BeamContext {
	fn say(&mut self, melody: Say);
}

impl BeamContext for BeamScope {
	fn say(&mut self, melody: Say) {
		self.melodies.push(melody)
	}
}
