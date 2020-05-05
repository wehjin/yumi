use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::sync::Arc;

use crate::hamt::frame::Frame;
use crate::hamt::reader::Reader;
use crate::hamt::root::Root;
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;
use crate::mem_file::EntryFile;

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use crate::hamt::data::{fixture::ZeroThenKeySlotIndexer};
	use crate::hamt::slot_indexer::SlotIndexer;
	use crate::hamt::writer::{WriteContext, Writer};
	use crate::mem_file::MemFile;

	struct WriteScope { transition_depth: usize }

	impl WriteContext for WriteScope {
		fn slot_indexer(&self, key: u32) -> Box<dyn SlotIndexer> {
			Box::new(ZeroThenKeySlotIndexer { key, transition_depth: self.transition_depth })
		}
	}

	#[ignore]
	#[test]
	fn double_write_double_level_collision_changes_read() {
		let mut scope = WriteScope { transition_depth: 2 };
		let cursor = Arc::new(MemFile::new());
		let mut writer = Writer::new(cursor, 0, 0);
		// First places value in empty slot of root-frame.
		writer.write(1, 10, &mut scope).unwrap();
		// Second finds occupied slot. Create 2 sub-frames before finding collision-free hash.
		writer.write(2, 20, &mut scope).unwrap();

		let reader = writer.reader().unwrap();
		let value1 = reader.read(&mut scope.slot_indexer(1)).unwrap();
		let value2 = reader.read(&mut scope.slot_indexer(2)).unwrap();
		assert_eq!((value1, value2), (Some(10), Some(20)));
	}

	#[test]
	fn triple_write_single_slot_changes_read() {
		let mut scope = WriteScope { transition_depth: 1 };
		let cursor = Arc::new(MemFile::new());
		let mut writer = Writer::new(cursor, 0, 0);
		// First places value in empty slot of root-frame.
		writer.write(1, 10, &mut scope).unwrap();
		{
			let reader = writer.reader().unwrap();
			let value1 = reader.read(&mut scope.slot_indexer(1)).unwrap();
			assert_eq!(value1, Some(10));
		}
		// Second finds occupied slot. Create sub-frame for first and second values.
		writer.write(2, 20, &mut scope).unwrap();
		{
			let reader = writer.reader().unwrap();
			let value1 = reader.read(&mut scope.slot_indexer(1)).unwrap();
			let value2 = reader.read(&mut scope.slot_indexer(2)).unwrap();
			assert_eq!((value1, value2), (Some(10), Some(20)));
		}
		// Third finds occupied slot. Places third value in empty slot of sub-frame.
		writer.write(3, 30, &mut scope).unwrap();
		{
			let reader = writer.reader().unwrap();
			let value1 = reader.read(&mut scope.slot_indexer(1)).unwrap();
			let value2 = reader.read(&mut scope.slot_indexer(2)).unwrap();
			let value3 = reader.read(&mut scope.slot_indexer(3)).unwrap();
			assert_eq!((value1, value2, value3), (Some(10), Some(20), Some(30)));
		}
	}

	#[test]
	fn single_write_changes_read() {
		let mut scope = WriteScope { transition_depth: 1 };
		let key = 0x00000001;

		let cursor = Arc::new(MemFile::new());
		let mut writer = Writer::new(cursor, 0, 0);
		writer.write(0x00000001, 17, &mut scope).unwrap();

		let reader = writer.reader().unwrap();
		let read = reader.read(&mut scope.slot_indexer(key)).unwrap();
		assert_eq!(read, Some(17));
	}
}

pub(crate) struct Writer {
	dest: Arc<dyn EntryFile>,
	root_pos: usize,
	root_mask: u32,
}

impl Writer {
	pub fn reader(&self) -> Result<Reader, Box<dyn Error>> {
		let dest = self.dest.clone();
		let len = dest.len();
		let (source, pos) = (dest, len?);
		let root_pos = self.root_pos;
		assert_eq!(root_pos, pos);
		Reader::new(source, root_pos, self.root_mask)
	}

	pub fn write(&mut self, key: u32, value: u32, ctx: &mut impl WriteContext) -> Result<(), Box<dyn Error>> {
		self.require_empty_high_bit(key)?;
		let mut reader = self.reader()?;
		let mut indexer = ctx.slot_indexer(key);
		let (mask, pos) = self.write_indexer(&mut indexer, 0, reader.root_pos, reader.root_mask, value, &mut reader)?;
		self.root_pos = pos;
		self.root_mask = mask;
		Ok(())
	}

	fn write_indexer(&mut self, indexer: &mut impl SlotIndexer, depth: usize, pos: usize, mask: u32, value: u32, reader: &mut Reader) -> Result<(u32, usize), Box<dyn Error>> {
		let key = indexer.key();
		let index = indexer.slot_index(depth);
		let frame = reader.read_frame(pos, mask)?;
		match &frame.slots[index as usize] {
			Slot::KeyValue(conflict_key, conflict_value) => {
				if *conflict_key == key {
					frame.with_value_slot(index, key, value).write(&mut self.dest)
				} else {
					let next_depth = depth + 1;
					let mut conflict_indexer = indexer.with_key(*conflict_key);
					let conflict_index = conflict_indexer.slot_index(next_depth);
					let next_index = indexer.slot_index(next_depth);
					if conflict_index == next_index {
						unimplemented!()
					} else {
						let sub_frame = Frame::empty()
							.with_value_slot(conflict_index, *conflict_key, *conflict_value)
							.with_value_slot(next_index, key, value);
						let (sub_mask, sub_pos) = sub_frame.write(&mut self.dest)?;
						let sub_pos = self.require_empty_high_bit(sub_pos as u32)?;
						let (parent_mask, parent_pos) = frame.with_ref_slot(index, sub_pos, sub_mask).write(&mut self.dest)?;
						Ok((parent_mask, parent_pos))
					}
				}
			}
			Slot::Root(root) => {
				match root {
					Root::PosMask(ref_pos, ref_mask) => {
						let (sub_mask, sub_pos) = self.write_indexer(indexer, depth + 1, *ref_pos as usize, *ref_mask, value, reader)?;
						let sub_pos = self.require_empty_high_bit(sub_pos as u32)?;
						let parent_frame = frame.with_ref_slot(index, sub_pos, sub_mask);
						let (parent_mask, parent_pos) = parent_frame.write(&mut self.dest)?;
						Ok((parent_mask, parent_pos))
					}
					Root::Frame(_) => {
						unimplemented!()
					}
				}
			}
			Slot::Empty => {
				let frame = frame.with_value_slot(index, key, value);
				let (mask, pos) = frame.write(&self.dest)?;
				Ok((mask, pos))
			}
		}
	}

	fn require_empty_high_bit(&self, n: u32) -> Result<u32, Box<dyn Error>> {
		if (n & 0x80000000) != 0 {
			Err(Box::new(io::Error::new(ErrorKind::InvalidData, "N exceeds 31 bits")))
		} else {
			Ok(n)
		}
	}

	pub fn new(dest: Arc<dyn EntryFile>, root_pos: usize, root_mask: u32) -> Self {
		Writer { dest: dest.clone(), root_pos, root_mask }
	}
}

pub(crate) trait WriteContext {
	fn slot_indexer(&self, key: u32) -> Box<dyn SlotIndexer>;
}
