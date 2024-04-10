#[cfg(test)]
mod tests {
	use crate::util::datom_tree::node_map;
	use crate::util::datom_tree::node_map::counting_mask;

	#[test]
	fn array_index_basic() {
		let map = 0b10u32;
		let key = 1u8;
		let index = node_map::array_index(key, map);
		assert_eq!(Some(0), index);
	}

	#[test]
	fn counting_mask_basic() {
		let masks = [
			counting_mask(1u8),
			counting_mask(2u8),
			counting_mask(3u8),
		];
		assert_eq!([0b0001, 0b0011, 0b0111], masks);
	}
}

pub fn array_index(key: u8, map: u32) -> Option<usize> {
	let flag = key_flag(key);
	if map & flag == flag {
		let masked_map = map & counting_mask(key);
		let count = masked_map.count_ones();
		Some(count as usize)
	} else {
		None
	}
}

fn counting_mask(key: u8) -> u32 {
	!(0xffffffffu32 << key)
}

pub fn key_flag(key: u8) -> u32 {
	0x1u32 << key
}

pub fn expand(map: u32) -> [Option<usize>; 32] {
	let mut result = [None; 32];
	let mut flag = 0x1u32;
	let mut count = 0;
	for i in 0..32 {
		if (map & flag) == flag {
			result[i] = Some(count);
			count += 1;
		}
		flag <<= 1;
	}
	result
}

pub fn compress(expanded: [bool; 32]) -> u32 {
	let mut flag = 0x1u32;
	let mut map = 0u32;
	for i in 0..32 {
		if expanded[i] {
			map |= flag;
		}
		flag <<= 1;
	}
	map
}
