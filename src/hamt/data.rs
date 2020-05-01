use std::io::Cursor;

pub fn byte_cursor() -> Cursor<Vec<u8>> {
	Cursor::new(Vec::new())
}

pub fn test_hash(subject: u32, index: usize, _prev: u8) -> u8 {
	(subject >> (5 * index as u32)) as u8
}

