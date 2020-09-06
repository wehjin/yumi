use std::io;
use std::io::ErrorKind;

use crate::diary;
use crate::hamt::frame::{SlotIndex, WriteSlot};
use crate::hamt::frame;
use crate::hamt::reader::Reader;
use crate::hamt::root::Root;
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;

#[cfg(test)]
mod tests {
	use std::error::Error;
	use std::path::PathBuf;

	use crate::diary::Diary;
	use crate::hamt::data::{fixture::ZeroThenKeySlotIndexer};
	use crate::hamt::reader::Reader;
	use crate::hamt::Root;
	use crate::hamt::slot_indexer::SlotIndexer;
	use crate::hamt::writer::Writer;

	#[test]
	fn double_write_multiple_collision_changes_read() -> Result<(), Box<dyn Error>> {
		let mut slot_indexer1 = ZeroThenKeySlotIndexer { key: 1, transition_depth: 3 };
		let mut slot_indexer2 = ZeroThenKeySlotIndexer { key: 2, transition_depth: 3 };
		let (root, path) = write_values(Root::ZERO, vec![
			// First write places value in empty slot of root-frame.
			(&mut slot_indexer1, 10),
			// Second write finds sub-frame in slot. But next 2 levels are collisions and creates 2 sub-frames before finding collision-free hash.
			(&mut slot_indexer2, 20),
		])?;
		let diary = Diary::load(&path)?;
		let mut diary_reader = diary.reader()?;
		let reader = Reader::new(root);
		let value1 = reader.read(&mut slot_indexer1, &mut diary_reader)?;
		let value2 = reader.read(&mut slot_indexer2, &mut diary_reader)?;
		assert_eq!((value1, value2), (Some(10), Some(20)));
		Ok(())
	}

	#[test]
	fn triple_write_single_slot_changes_read() -> Result<(), Box<dyn Error>> {
		let mut slot_indexer1 = ZeroThenKeySlotIndexer { key: 1, transition_depth: 1 };
		let mut slot_indexer2 = ZeroThenKeySlotIndexer { key: 2, transition_depth: 1 };
		let mut slot_indexer3 = ZeroThenKeySlotIndexer { key: 3, transition_depth: 1 };
		let (new_root, path) = write_values(Root::ZERO, vec![
			// First places value in empty slot of root-frame.
			(&mut slot_indexer1, 10),
			// Second finds slot occupied by first value. Create sub-frame for first and second values.
			(&mut slot_indexer2, 20),
			// Third finds slot occupied by sub-frame. Places third value in empty slot of sub-frame.
			(&mut slot_indexer3, 30),
		])?;
		let diary = Diary::load(&path)?;
		let mut diary_reader = diary.reader()?;
		let reader = Reader::new(new_root);
		let value1 = reader.read(&mut slot_indexer1, &mut diary_reader)?;
		let value2 = reader.read(&mut slot_indexer2, &mut diary_reader)?;
		let value3 = reader.read(&mut slot_indexer3, &mut diary_reader)?;
		assert_eq!((value1, value2, value3), (Some(10), Some(20), Some(30)));
		Ok(())
	}

	#[test]
	fn single_write_changes_read() -> Result<(), Box<dyn Error>> {
		let mut slot_indexer = ZeroThenKeySlotIndexer { key: 1, transition_depth: 1 };
		let (new_root, path) = write_values(Root::ZERO, vec![
			(&mut slot_indexer, 17)
		])?;
		let diary = Diary::load(&path)?;
		let mut diary_reader = diary.reader()?;
		let reader = Reader::new(new_root);
		let reading = reader.read(&mut slot_indexer, &mut diary_reader)?;
		assert_eq!(reading, Some(17));
		Ok(())
	}

	fn write_values(root: Root, tasks: Vec<(&mut impl SlotIndexer, u32)>) -> Result<(Root, PathBuf), Box<dyn Error>> {
		let diary = Diary::temp()?;
		let mut diary_writer = diary.writer()?;
		let new_root = {
			let mut writer = Writer::new(root, &mut diary_writer);
			for (slot_indexer, value) in tasks {
				writer.write(value, slot_indexer)?;
			}
			writer.root
		};
		diary.commit(diary_writer.end_size());
		Ok((new_root, diary.file_path.to_owned()))
	}
}

// TODO Move diary_writer from struct to fn arg.
pub(crate) struct Writer<'a> {
	root: Root,
	diary_writer: &'a mut diary::Writer,
}

enum WriteRoot {
	ReviseWithValue(Root, SlotIndex, Slot),
	ReviseWithSubRoot(Root, SlotIndex),
	AddWithValues(SlotIndex, Slot, SlotIndex, Slot),
	AddWithSubRoot(SlotIndex),
}

impl<'a> Writer<'a> {
	pub fn write(&mut self, value: u32, slot_indexer: &mut impl SlotIndexer) -> io::Result<Root> {
		require_empty_high_bit(slot_indexer.key())?;
		let mut diary_reader = self.diary_writer.reader()?;
		let revisions = {
			let reader = Reader::new(self.root);
			let mut revisions = Vec::new();
			let mut depth = 0;
			let mut root = self.root;
			let mut done = false;
			while !done {
				let slot_index = SlotIndex::at(slot_indexer.slot_index(depth) as usize);
				match reader.read_slot(root, slot_index, &mut diary_reader)? {
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
							while resolution_indices == None {
								resolution_depth += 1;
								let defender_index = SlotIndex::at(defender_indexer.slot_index(resolution_depth) as usize);
								let attacker_index = SlotIndex::at(slot_indexer.slot_index(resolution_depth) as usize);
								if attacker_index == defender_index {
									revisions.push(WriteRoot::AddWithSubRoot(attacker_index))
								} else {
									resolution_indices = Some((attacker_index, defender_index));
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
		Ok(self.root)
	}
	pub fn new(root: Root, diary_writer: &'a mut diary::Writer) -> Self { Writer { root, diary_writer } }
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
