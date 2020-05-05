use std::hash::Hasher;

use crate::util;

#[cfg(test)]
mod tests {
	use crate::hamt::hasher::universal;

	#[test]
	fn keys_under_256_hash_to_themselves() {
		let hashes = (universal(10, 1), universal(10, 382423));
		assert_eq!(hashes, (10, 10));
	}

	#[test]
	fn keys_at_over_256_are_change_per_depth() {
		let (h1, h2) = (universal(310, 1), universal(310, 2));
		assert_ne!(h1, 310);
		assert_ne!(h1, h2);
	}
}

pub(crate) fn universal(key: u32, level: u32) -> u32 {
	let mut key_parts = [0u8; 4];
	util::big_end_first_4(key, &mut key_parts);
	let mut hasher = UniversalHasher::new(level as u64);
	hasher.write(&key_parts);
	hasher.finish() as u32
}

pub struct UniversalHasher {
	level: u64,
	a: u32,
	hash: u32,
}

impl UniversalHasher {
	pub fn new(level: u64) -> Self {
		UniversalHasher { level, a: 31415, hash: 0 }
	}
}

impl Hasher for UniversalHasher {
	fn finish(&self) -> u64 { self.hash as u64 }

	fn write(&mut self, bytes: &[u8]) {
		let level = self.level;
		bytes.iter().for_each(|byte| {
			let a = self.a as u64;
			self.hash = (a * self.hash as u64 * level + (*byte as u64)) as u32;
			self.a = (a * B) as u32;
		});
	}
}

static B: u64 = 27183;
