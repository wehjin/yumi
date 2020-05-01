use std::io;
use std::io::{Cursor, ErrorKind, Write};
use std::ops::Deref;
use std::rc::Rc;

use crate::hamt::reader::{Reader, Source};

#[cfg(test)]
mod tests {
	use crate::hamt::data::{byte_cursor, test_hash};
	use crate::hamt::writer::Writer;

	#[test]
	fn write_changes_read() {
		let mut writer = Writer::new(byte_cursor(), 0, test_hash);
		let key = 0x00000001;
		writer.write(0x00000001, 17).unwrap();
		let reader = writer.reader().unwrap();
		let value = reader.read(key);
		assert_eq!(value, Some(17));
	}
}

pub(crate) struct Writer {
	dest: Box<dyn Dest>,
	root_pos: u64,
	hasher: Rc<dyn Fn(u32, usize, u8) -> u8>,
}

impl Writer {
	pub fn reader(&self) -> io::Result<Reader> {
		let (source, pos) = self.dest.as_source();
		Reader::new(source, pos, self.hasher.to_owned())
	}

	pub fn write(&mut self, key: u32, value: u32) -> io::Result<()> {
		if (value & 0x80000000) > 0 {
			Err(io::Error::new(ErrorKind::InvalidData, "Value is large than 31 bits"))
		} else {
			let root = {
				let reader = self.reader()?;
				reader.root_frame
			};
			let slot_index = (*self.hasher)(key, 0, 0);
			let frame = root.update_value(slot_index, key, value);
			let bytes_written = frame.write(&mut self.dest)?;
			self.root_pos += bytes_written;
			Ok(())
		}
	}

	pub fn new(dest: impl Dest, root_pos: u64, hasher: impl Fn(u32, usize, u8) -> u8 + 'static) -> Self {
		Writer {
			dest: Box::new(dest),
			root_pos,
			hasher: Rc::new(hasher),
		}
	}
}

pub trait Dest: Write + 'static {
	fn as_source(&self) -> (Box<dyn Source>, u64);
}

impl Dest for Cursor<Vec<u8>> {
	fn as_source(&self) -> (Box<dyn Source>, u64) {
		let vec = self.get_ref().to_vec();
		let max_pos = vec.len() as u64;
		let cursor = Cursor::new(vec);
		(Box::new(cursor), max_pos)
	}
}

impl Dest for Box<dyn Dest> {
	fn as_source(&self) -> (Box<dyn Source>, u64) {
		self.deref().as_source()
	}
}

