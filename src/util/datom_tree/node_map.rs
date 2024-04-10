pub fn array_index(key: u8, map: u32) -> usize {
	let counting_mask = 0xffffffffu32 << key;
	(map & counting_mask.count_ones()) as usize
}

pub fn map_entry(key: u8) -> u32 {
	0x1u32 << key
}
