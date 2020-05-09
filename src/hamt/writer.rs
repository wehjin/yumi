use std::error::Error;
use std::io;
use std::io::ErrorKind;
use std::net::Shutdown::Write;
use std::ops::Deref;
use std::sync::Arc;

use crate::diary;
use crate::hamt::frame::{Frame, SlotIndex, WriteSlot};
use crate::hamt::frame;
use crate::hamt::reader::{Reader, Reader2};
use crate::hamt::root::Root;
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;
use crate::mem_file::EntryFile;
use crate::util::io_error_of_box;

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use crate::hamt::data::{fixture::ZeroThenKeySlotIndexer};
	use crate::hamt::Root;
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
		let mut writer = Writer::new(cursor, Root::ZERO);
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
		let mut writer = Writer::new(cursor, Root::ZERO);
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

	// TODO Convert tests to Writer2.
	#[test]
	fn single_write_changes_read() {
		let mut scope = WriteScope { transition_depth: 1 };
		let key = 0x00000001;

		let cursor = Arc::new(MemFile::new());
		let mut writer = Writer::new(cursor, Root::ZERO);
		writer.write(0x00000001, 17, &mut scope).unwrap();

		let reader = writer.reader().unwrap();
		let read = reader.read(&mut scope.slot_indexer(key)).unwrap();
		assert_eq!(read, Some(17));
	}
}

pub(crate) struct Writer2<'a> {
	root: Root,
	diary_writer: &'a mut diary::Writer,
	build_diary_reader: Box<dyn Fn() -> diary::Reader>,
}

enum WriteRoot {
	ReviseWithValue(Root, SlotIndex, Slot),
	ReviseWithSubRoot(Root, SlotIndex),
	AddWithValues(SlotIndex, Slot, SlotIndex, Slot),
	AddWithSubRoot(SlotIndex),
}

impl<'a> Writer2<'a> {
	pub fn write(&mut self, value: u32, slot_indexer: &mut impl SlotIndexer) -> io::Result<()> {
		require_empty_high_bit(slot_indexer.key())?;
		let mut diary_reader = self.build_diary_reader.deref()();
		let revisions = {
			let mut revisions = Vec::new();
			let mut reader = Reader2::new(self.root, &mut diary_reader);
			let mut depth = 0;
			let mut root = self.root;
			let mut done = false;
			let slot_index = SlotIndex::at(slot_indexer.slot_index(depth) as usize);
			match reader.read_slot(root, slot_index)? {
				Slot::Root(sub_root) => {
					revisions.push(WriteRoot::ReviseWithSubRoot(root, slot_index));
					root = sub_root;
					depth += 1;
				}
				Slot::KeyValue(defender_key, defender_value) => {
					let attacker_key = slot_indexer.key();
					if defender_key == attacker_key {
						revisions.push(WriteRoot::ReviseWithValue(root, slot_index, Slot::KeyValue(defender_key, value)));
						done = true;
					} else {
						revisions.push(WriteRoot::ReviseWithSubRoot(root, slot_index));
						let mut resolution_indices = None;
						let mut defender_indexer = slot_indexer.with_key(defender_key);
						let mut resolution_depth = depth;
						loop {
							resolution_depth += 1;
							let defender_index = SlotIndex::at(defender_indexer.slot_index(resolution_depth) as usize);
							let attacker_index = SlotIndex::at(slot_indexer.slot_index(resolution_depth) as usize);
							if attacker_index == defender_index {
								revisions.push(WriteRoot::AddWithSubRoot(attacker_index))
							} else {
								resolution_indices = Some((attacker_index, defender_index));
								break;
							}
						}
						let (attacker_index, defender_index) = resolution_indices.unwrap();
						revisions.push(WriteRoot::AddWithValues(
							attacker_index, Slot::KeyValue(attacker_key, value),
							defender_index, Slot::KeyValue(defender_key, defender_value),
						));
						done = true;
					}
				}
				Slot::Empty => {
					revisions.push(WriteRoot::ReviseWithValue(root, slot_index, Slot::KeyValue(slot_indexer.key(), value)));
					done = true;
				}
			}
			revisions.reverse();
			revisions
		};
		let mut writer = frame::Writer::new(self.diary_writer);
		let mut current_root = Root::ZERO;
		for revision in revisions {
			match revision {
				WriteRoot::ReviseWithValue(old_root, slot_index, new_slot) => {
					let mut frame_reader = frame::Reader::new(old_root, &mut diary_reader)?;
					let new_root = writer.write_revised_root(
						WriteSlot { slot: new_slot, slot_index },
						&mut frame_reader,
					)?;
					current_root = require_empty_high_bit_in_position(new_root)?;
				}
				WriteRoot::ReviseWithSubRoot(old_root, slot_index) => {
					let new_slot = Slot::Root(current_root);
					let mut frame_reader = frame::Reader::new(old_root, &mut diary_reader)?;
					let new_root = writer.write_revised_root(
						WriteSlot { slot: new_slot, slot_index },
						&mut frame_reader,
					)?;
					current_root = require_empty_high_bit_in_position(new_root)?;
				}
				WriteRoot::AddWithValues(index_a, new_slot_a, index_b, new_slot_b) => {
					let new_root = writer.write_root_with_slots(
						WriteSlot { slot: new_slot_a, slot_index: index_a },
						WriteSlot { slot: new_slot_b, slot_index: index_b },
					)?;
					current_root = require_empty_high_bit_in_position(new_root)?;
				}
				WriteRoot::AddWithSubRoot(slot_index) => {
					let new_slot = Slot::Root(current_root);
					let new_root = writer.write_root_with_slot(
						WriteSlot { slot: new_slot, slot_index }
					)?;
					current_root = require_empty_high_bit_in_position(new_root)?;
				}
			}
		}
		self.root = current_root;
		Ok(())
	}
	pub fn new(root: Root, diary_writer: &'a mut diary::Writer, build_diary_reader: impl Fn() -> diary::Reader + 'static) -> Self {
		Writer2 { root, diary_writer, build_diary_reader: Box::new(build_diary_reader) }
	}
}

pub(crate) struct Writer {
	dest: Arc<dyn EntryFile>,
	root: Root,
}

impl Writer {
	pub fn reader(&self) -> Result<Reader, Box<dyn Error>> {
		Reader::new(self.dest.clone(), self.root)
	}

	pub fn write(&mut self, key: u32, value: u32, ctx: &mut impl WriteContext) -> io::Result<()> {
		require_empty_high_bit(key)?;
		let mut reader = self.reader().map_err(|e| {
			io::Error::new(ErrorKind::Other, e.to_string())
		})?;
		let mut indexer = ctx.slot_indexer(key);
		let root = self.write_indexer(&mut indexer, 0, reader.root, value, &mut reader).map_err(|e| {
			io::Error::new(ErrorKind::Other, e.to_string())
		})?;
		self.root = root;
		Ok(())
	}

	fn write_indexer(&mut self, indexer: &mut impl SlotIndexer, depth: usize, root: Root, value: u32, reader: &mut Reader) -> io::Result<Root> {
		let key = indexer.key();
		let index = indexer.slot_index(depth);
		let frame = reader.read_frame(root).map_err(io_error_of_box)?;
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
						let sub_root: Root = sub_frame.write(&mut self.dest)?;
						let sub_root = self.require_31_bit_position(sub_root)?;
						let parent_root = frame.with_ref_slot(index, sub_root).write(&mut self.dest)?;
						Ok(parent_root)
					}
				}
			}
			Slot::Root(root) => {
				let sub_root: Root = self.write_indexer(indexer, depth + 1, *root, value, reader)?;
				let sub_root = self.require_31_bit_position(sub_root)?;
				let parent_frame = frame.with_ref_slot(index, sub_root);
				let parent_root = parent_frame.write(&mut self.dest)?;
				Ok(parent_root)
			}
			Slot::Empty => {
				let frame = frame.with_value_slot(index, key, value);
				let root = frame.write(&self.dest)?;
				Ok(root)
			}
		}
	}

	fn require_31_bit_position(&self, root: Root) -> io::Result<Root> {
		require_empty_high_bit(root.pos).map(|_| root)
	}

	pub fn root(&self) -> Root { self.root }
	pub fn new(dest: Arc<dyn EntryFile>, root: Root) -> Self { Writer { dest: dest.clone(), root } }
}

pub(crate) trait WriteContext {
	fn slot_indexer(&self, key: u32) -> Box<dyn SlotIndexer>;
}

fn require_empty_high_bit_in_position(root: Root) -> io::Result<Root> {
	require_empty_high_bit(root.pos).map(|_| root)
}

fn require_empty_high_bit(n: u32) -> io::Result<u32> {
	if (n & 0x80000000) != 0 {
		Err(io::Error::new(ErrorKind::InvalidData, "N exceeds 31 bits"))
	} else {
		Ok(n)
	}
}
