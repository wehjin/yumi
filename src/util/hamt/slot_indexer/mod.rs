use std::ops::{Deref, DerefMut};

use crate::util::hamt::hasher::universal;

#[cfg(test)]
mod tests {
	use std::collections::HashSet;

	use crate::util::hamt::slot_indexer::{SlotIndexer, UniversalSlotPicker};

	#[test]
	fn keys_under_256_cyclically_map_to_themselves() {
		let mut keyhash = UniversalSlotPicker::new(10);
		let depths = [0, 1, 2, 3, 4, 5, 6, 7];
		let indices = depths.iter().map(|it| keyhash.slot_index(*it)).collect::<Vec<_>>();
		assert_eq!(indices, vec![10, 0, 0, 0, 0, 0, 10, 0]);
		indices.iter().for_each(|it| { assert!(*it < 32); })
	}

	#[test]
	fn keys_at_over_256_map_to_differing_values_per_depth() {
		let mut keyhash = UniversalSlotPicker::new(256);
		let depths = [0, 1, 2, 3, 4, 5, 6, 7];
		let indices = depths.iter().map(|it| keyhash.slot_index(*it)).collect::<HashSet<_>>();
		assert!(indices.len() > 5);
		indices.iter().for_each(|it| { assert!(*it < 32); })
	}
}

pub trait SlotIndexer {
	fn key(&self) -> u32;
	fn slot_index(&mut self, depth: usize) -> u8;
	fn with_key(&self, key: u32) -> Box<dyn SlotIndexer>;
}

impl SlotIndexer for Box<dyn SlotIndexer> {
	fn key(&self) -> u32 { self.deref().key() }
	fn slot_index(&mut self, depth: usize) -> u8 { self.deref_mut().slot_index(depth) }
	fn with_key(&self, key: u32) -> Box<dyn SlotIndexer> { self.deref().with_key(key) }
}

pub(crate) struct UniversalSlotPicker {
	key: u32,
	hashes: Vec<u32>,
}

impl SlotIndexer for UniversalSlotPicker {
	fn key(&self) -> u32 { self.key }
	fn slot_index(&mut self, depth: usize) -> u8 {
		let hashes_index = depth / LEVELS_PER_HASH;
		self.prepare_hashes(hashes_index);
		let hash = &self.hashes[hashes_index];
		let shift = (depth % LEVELS_PER_HASH) * BITS_PER_LEVEL;
		let slot_index = ((hash >> shift) & LEVEL_MASK) as u8;
		slot_index
	}

	fn with_key(&self, key: u32) -> Box<dyn SlotIndexer> {
		Box::new(UniversalSlotPicker { key, hashes: Vec::new() })
	}
}

impl UniversalSlotPicker {
	pub fn new(key: u32) -> Self {
		UniversalSlotPicker { key, hashes: Vec::new() }
	}

	fn prepare_hashes(&mut self, hashes_index: usize) {
		if hashes_index >= self.hashes.len() {
			let max_index = hashes_index + 1;
			for index in self.hashes.len()..max_index {
				let hash = universal(self.key, (index + 1) as u32);
				self.hashes.push(hash);
			}
		}
	}
}

static LEVEL_MASK: u32 = 0x1f;
static LEVELS_PER_HASH: usize = BITS_PER_HASH / BITS_PER_LEVEL;
static BITS_PER_HASH: usize = BITS_PER_KEY;
static BITS_PER_LEVEL: usize = 5;
static BITS_PER_KEY: usize = 32;
