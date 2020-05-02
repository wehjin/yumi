#[cfg(test)]
use std::io::Cursor;

#[cfg(test)]
pub fn byte_cursor() -> Cursor<Vec<u8>> {
	Cursor::new(Vec::new())
}

#[cfg(test)]
pub mod fixture {
	#[cfg(test)]
	use crate::hamt::slot_indexer::SlotIndexer;

	pub struct ZeroThenKeySlotIndexer {
		pub key: u32,
		pub transition_depth: usize,
	}

	impl SlotIndexer for ZeroThenKeySlotIndexer {
		fn key(&self) -> u32 { self.key }
		fn slot_index(&mut self, depth: usize) -> u8 {
			if depth < self.transition_depth {
				0
			} else {
				(self.key as u8) % 32
			}
		}

		fn with_key(&self, key: u32) -> Box<dyn SlotIndexer> {
			Box::new(ZeroThenKeySlotIndexer { key, transition_depth: self.transition_depth })
		}
	}
}
