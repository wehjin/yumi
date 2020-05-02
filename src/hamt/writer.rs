use std::io;
use std::io::{Cursor, ErrorKind, Write};
use std::ops::Deref;

use crate::hamt::frame::Frame;
use crate::hamt::reader::{Reader, Source};
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;

#[cfg(test)]
mod tests {
	use crate::hamt::data::{byte_cursor, fixture::ZeroThenKeySlotIndexer};
	use crate::hamt::slot_indexer::SlotIndexer;
	use crate::hamt::writer::{WriteContext, Writer};

	struct WriteScope;

	impl WriteContext for WriteScope {
		fn slot_indexer(&self, key: u32) -> Box<dyn SlotIndexer> {
			Box::new(ZeroThenKeySlotIndexer { key })
		}
	}

	#[test]
	fn double_write_single_slot_changes_read() {
		let mut scope = WriteScope {};
		let cursor = byte_cursor();
		let mut writer = Writer::new(cursor, 0, 0);
		writer.write(1, 10, &mut scope).unwrap();
		writer.write(2, 20, &mut scope).unwrap();

		let mut reader = writer.reader().unwrap();
		let value1 = reader.read(&mut scope.slot_indexer(1)).unwrap();
		let value2 = reader.read(&mut scope.slot_indexer(2)).unwrap();
		assert_eq!((value1, value2), (Some(10), Some(20)));
	}

	#[test]
	fn single_write_changes_read() {
		let mut scope = WriteScope {};
		let key = 0x00000001;

		let mut writer = Writer::new(byte_cursor(), 0, 0);
		writer.write(0x00000001, 17, &mut scope).unwrap();

		let mut reader = writer.reader().unwrap();
		let read = reader.read(&mut scope.slot_indexer(key)).unwrap();
		assert_eq!(read, Some(17));
	}
}

pub(crate) struct Writer {
	dest: Box<dyn Dest>,
	root_pos: usize,
	root_mask: u32,
}

impl Writer {
	pub fn reader(&self) -> io::Result<Reader> {
		let (source, _) = self.dest.as_source();
		Reader::new(source, self.root_pos, self.root_mask)
	}

	pub fn write(&mut self, key: u32, value: u32, ctx: &mut impl WriteContext) -> io::Result<()> {
		self.require_empty_high_bit(key)?;
		let frame = self.reader()?.root_frame;
		let mut slot_indexer = ctx.slot_indexer(key);
		let depth = 0;
		let slot_index = slot_indexer.slot_index(depth);
		let (mask, bytes_written) = match frame.slots[slot_index as usize] {
			Slot::Empty => frame.with_value(slot_index, key, value).write(&mut self.dest)?,
			Slot::Value { key: conflict_key, value: conflict_value } => {
				if conflict_key == key {
					frame.with_value(slot_index, key, value).write(&mut self.dest)?
				} else {
					let next_depth = depth + 1;
					let mut conflict_indexer = ctx.slot_indexer(conflict_key);
					let conflict_index = conflict_indexer.slot_index(next_depth);
					let next_index = slot_indexer.slot_index(next_depth);
					if conflict_index == next_index {
						unimplemented!()
					} else {
						let resolution_frame = Frame::clear()
							.with_value(conflict_index, conflict_key, conflict_value)
							.with_value(next_index, key, value);
						let (resolution_mask, resolution_bytes_written) = resolution_frame.write(&mut self.dest)?;
						let resolution_pos = self.require_empty_high_bit((self.root_pos + resolution_bytes_written) as u32)?;
						let (parent_mask, parent_bytes_written) = frame.with_pos(slot_index, resolution_pos, resolution_mask).write(&mut self.dest)?;
						(parent_mask, parent_bytes_written + resolution_bytes_written)
					}
				}
			}
			Slot::Ref { .. } => unimplemented!()
		};
		self.root_pos += bytes_written;
		self.root_mask = mask;
		Ok(())
	}

	fn require_empty_high_bit(&self, n: u32) -> io::Result<u32> {
		if (n & 0x80000000) != 0 {
			Err(io::Error::new(ErrorKind::InvalidData, "N exceeds 31 bits"))
		} else {
			Ok(n)
		}
	}

	pub fn new(dest: impl Dest, root_pos: usize, root_mask: u32) -> Self {
		Writer { dest: Box::new(dest), root_pos, root_mask }
	}
}

pub(crate) trait WriteContext {
	fn slot_indexer(&self, key: u32) -> Box<dyn SlotIndexer>;
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

