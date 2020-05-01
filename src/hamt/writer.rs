use std::io;
use std::io::{Cursor, ErrorKind, Write};
use std::ops::Deref;

use crate::hamt::reader::{Reader, Source};
use crate::hamt::slot_indexer::SlotIndexer;

#[cfg(test)]
mod tests {
	use crate::hamt::data::{byte_cursor, fixture::DepthSlotIndexer};
	use crate::hamt::slot_indexer::SlotIndexer;
	use crate::hamt::writer::{WriteContext, Writer};

	struct WriteScope;

	impl WriteContext for WriteScope {
		fn slot_indexer(&self, key: u32) -> Box<dyn SlotIndexer> {
			Box::new(DepthSlotIndexer { key })
		}
	}

	#[test]
	fn write_changes_read() {
		let mut writer = Writer::new(byte_cursor(), 0);
		let key = 0x00000001;

		let mut scope = WriteScope {};
		writer.write(0x00000001, 17, &mut scope).unwrap();

		let reader = writer.reader().unwrap();
		let mut slot_indexer = DepthSlotIndexer { key };
		let value = reader.read(&mut slot_indexer);
		assert_eq!(value, Some(17));
	}
}

pub(crate) trait WriteContext {
	fn slot_indexer(&self, key: u32) -> Box<dyn SlotIndexer>;
}

pub(crate) struct Writer {
	dest: Box<dyn Dest>,
	root_pos: u64,
}

impl Writer {
	pub fn reader(&self) -> io::Result<Reader> {
		let (source, pos) = self.dest.as_source();
		Reader::new(source, pos)
	}

	pub fn write(&mut self, key: u32, value: u32, ctx: &mut impl WriteContext) -> io::Result<()> {
		if (value & 0x80000000) > 0 {
			Err(io::Error::new(ErrorKind::InvalidData, "Value is large than 31 bits"))
		} else {
			let root = {
				let reader = self.reader()?;
				reader.root_frame
			};
			let mut slot_indexer = ctx.slot_indexer(key);
			let slot_index = slot_indexer.slot_index(0);
			let frame = root.update_value(slot_index, key, value);
			let bytes_written = frame.write(&mut self.dest)?;
			self.root_pos += bytes_written;
			Ok(())
		}
	}

	pub fn new(dest: impl Dest, root_pos: u64) -> Self {
		Writer { dest: Box::new(dest), root_pos }
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

