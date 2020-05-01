use crate::hamt::util;

#[cfg(test)]
mod tests {
	use crate::hamt::keyhash::hash::universal;

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
	let mut a: u32 = 31415;
	let mut hash: u32 = 0;
	let level = level as u64;
	key_parts.iter().for_each(|part| {
		hash = (a as u64 * hash as u64 * level + (*part as u64)) as u32;
		a = (a as u64 * B) as u32;
	});
	hash
}

static B: u64 = 27183;
