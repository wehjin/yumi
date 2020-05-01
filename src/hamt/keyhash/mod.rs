use std::borrow::Borrow;
use std::cell::Cell;

use crate::hamt::util;
use crate::hamt::util::big_end_first_4;

mod hash;

#[cfg(test)]
mod tests {
	use std::collections::HashSet;

	use crate::hamt::keyhash::Keyhash;

	#[test]
	fn keys_under_256_cyclically_map_to_themselves() {
		let mut keyhash = Keyhash::new(10);
		let depths = [0, 1, 2, 3, 4, 5, 6, 7];
		let indices = depths.iter().map(|it| keyhash.slot_index(*it)).collect::<Vec<_>>();
		assert_eq!(indices, vec![10, 0, 0, 0, 0, 0, 10, 0]);
	}

	#[test]
	fn keys_at_over_256_map_to_different_values_per_depth() {
		let mut keyhash = Keyhash::new(256);
		let depths = [0, 1, 2, 3, 4, 5, 6, 7];
		let indices = depths.iter().map(|it| keyhash.slot_index(*it)).collect::<HashSet<_>>();
		assert_eq!(indices.len(), depths.len());
	}
}

pub(crate) struct Keyhash {
	key: u32,
	hashes: Vec<u32>,
}

impl Keyhash {
	pub fn new(key: u32) -> Self {
		Keyhash { key, hashes: Vec::new() }
	}

	pub fn slot_index(&mut self, depth: usize) -> u8 {
		let hashes_index = depth / LEVELS_PER_HASH;
		self.prepare_hashes(hashes_index);
		let hash = &self.hashes[hashes_index];
		let shift = (depth % LEVELS_PER_HASH) * BITS_PER_LEVEL;
		((hash >> shift) & LEVEL_MASK) as u8
	}

	fn prepare_hashes(&mut self, hashes_index: usize) {
		if hashes_index >= self.hashes.len() {
			let max_index = hashes_index + 1;
			for index in self.hashes.len()..max_index {
				let hash = hash::universal(self.key, (index + 1) as u32);
				self.hashes.push(hash);
			}
		}
	}
}

static LEVEL_MASK: u32 = 0x1ffff;
static LEVELS_PER_HASH: usize = BITS_PER_HASH / BITS_PER_LEVEL;
static BITS_PER_HASH: usize = BITS_PER_KEY;
static BITS_PER_LEVEL: usize = 5;
static BITS_PER_KEY: usize = 32;
