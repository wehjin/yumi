use std::io::{Cursor, Read, Seek};

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::hamt::{data, Reader};

	#[test]
	fn empty_source_produces_none() {
		let source = Cursor::new(Vec::new());
		let reader = Reader::new(source, 0);
		let keys = [[1, 2, 3], [3, 6, 0], [8, 22, 11]];
		keys.iter().for_each(|key| {
			let value = reader.read(key);
			assert_eq!(value, None)
		});
	}
}

pub struct Reader {
	source: Box<dyn Source>,
	tip: usize,
}

pub trait Source: Seek + Read + 'static {}

impl Source for Cursor<Vec<u8>> {}

impl Reader {
	pub fn new(source: impl Source, tip: usize) -> Self {
		Reader { source: Box::new(source), tip }
	}

	pub fn read(&self, key: &[u8]) -> Option<usize> {
		if self.tip == 0 {
			None
		} else {
			None
		}
	}
}
