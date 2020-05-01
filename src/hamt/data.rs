#[cfg(test)]
use std::io::Cursor;

#[cfg(test)]
pub fn byte_cursor() -> Cursor<Vec<u8>> {
	Cursor::new(Vec::new())
}

#[cfg(test)]
pub fn test_hash(subject: u32, index: usize, _prev: u8) -> u8 {
	(subject >> (5 * index as u32)) as u8
}

#[cfg(test)]
pub mod fixture {
	#[cfg(test)]
	use crate::hamt::slot_indexer::SlotIndexer;

	pub struct DepthSlotIndexer { pub key: u32 }

	impl SlotIndexer for DepthSlotIndexer {
		fn key(&self) -> u32 { self.key }
		fn slot_index(&mut self, depth: usize) -> u8 {
			(depth % 32) as u8
		}

		fn with_key(&self, key: u32) -> Box<dyn SlotIndexer> {
			Box::new(DepthSlotIndexer { key })
		}
	}
}
