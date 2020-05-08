use std::collections::HashMap;
use std::io;

use crate::diary;
use crate::hamt::{Root, slot};
use crate::hamt::frame::SlotIndex;
use crate::hamt::slot::Slot;

#[cfg(test)]
mod tests {
	use std::error::Error;
	use std::path::Path;

	use crate::diary::Diary;
	use crate::hamt::{frame, Root};
	use crate::hamt::frame::read_write::WriteSlot;
	use crate::hamt::frame::SlotIndex;
	use crate::hamt::slot::Slot;

// TODO Write multiple slots with optional prototype.

	#[test]
	fn sub_root_with_two_slots() -> Result<(), Box<dyn Error>> {
		let write_slot1 = WriteSlot { slot: Slot::KeyValue(1, 10), slot_index: SlotIndex::at(1) };
		let write_slot7 = WriteSlot { slot: Slot::KeyValue(7, 70), slot_index: SlotIndex::at(7) };
		let (path, root) = {
			let diary = Diary::temp()?;
			let mut diary_writer = diary.writer()?;
			let mut writer = frame::Writer::new(&mut diary_writer);
			let root = writer.write_root_with_slots(write_slot1, write_slot7)?;
			(diary.file_path, root)
		};
		let mut slots = [None; 32];
		slots[write_slot1.slot_index.as_usize()] = Some(write_slot1.slot);
		slots[write_slot7.slot_index.as_usize()] = Some(write_slot7.slot);
		assert_slots(&path, root, &slots)
	}

	#[test]
	fn sub_root_with_one_slot() -> Result<(), Box<dyn Error>> {
		let write_slot = WriteSlot { slot: Slot::KeyValue(3, 30), slot_index: SlotIndex::at(3) };
		let (path, root) = {
			let diary = Diary::temp()?;
			let mut diary_writer = diary.writer()?;
			let mut writer = frame::Writer::new(&mut diary_writer);
			let root = writer.write_root_with_slot(write_slot)?;
			(diary.file_path, root)
		};
		let mut slots = [None; 32];
		slots[write_slot.slot_index.as_usize()] = Some(write_slot.slot);
		assert_slots(&path, root, &slots)
	}

	fn assert_slots(path: &Path, root: Root, slots: &[Option<Slot>; 32]) -> Result<(), Box<dyn Error>> {
		let diary = Diary::load(&path)?;
		let mut diary_reader = diary.reader()?;
		let mut reader = frame::Reader::new(root, &mut diary_reader)?;
		for n in 0..32 {
			let slot_index = SlotIndex::at(n);
			reader.seek(slot_index)?;
			let reading = reader.read()?;
			assert_eq!(reading, &slots[n]);
		}
		Ok(())
	}

	#[test]
	fn read_empty() -> Result<(), Box<dyn Error>> {
		let diary = Diary::temp()?;
		let mut diary_reader = diary.reader()?;
		let reader = frame::Reader::new(Root::ZERO, &mut diary_reader)?;
		let reading = reader.read().unwrap();
		assert_eq!(reading, &None);
		Ok(())
	}
}

pub(crate) struct Writer<'a> {
	slot_writer: slot::Writer<'a>,
}

impl<'a> Writer<'a> {
	pub fn write_root_with_slots(&mut self, write_slot_a: WriteSlot, write_slot_b: WriteSlot) -> io::Result<Root> {
		debug_assert_ne!(write_slot_a.slot_index, write_slot_b.slot_index);
		let (first_write, second_write) = if write_slot_a.slot_index < write_slot_b.slot_index {
			(write_slot_a, write_slot_b)
		} else {
			(write_slot_b, write_slot_a)
		};
		let (pos, _size) = self.slot_writer.write(&first_write.slot)?;
		self.slot_writer.write(&second_write.slot)?;
		let root_mask = first_write.slot_index.as_mask() | second_write.slot_index.as_mask();
		Ok(Root { pos: pos.u32(), mask: root_mask })
	}

	pub fn write_root_with_slot(&mut self, write_slot: WriteSlot) -> io::Result<Root> {
		let (pos, _size) = self.slot_writer.write(&write_slot.slot)?;
		Ok(Root { pos: pos.u32(), mask: write_slot.slot_index.as_mask() })
	}
	pub fn new(diary_writer: &'a mut diary::Writer) -> Self { Writer { slot_writer: slot::Writer::new(diary_writer) } }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct WriteSlot {
	pub slot: Slot,
	pub slot_index: SlotIndex,
}

pub(crate) struct Reader<'a> {
	root: Root,
	slot_reader: slot::Reader<'a>,
}

impl<'a> Reader<'a> {
	pub fn seek(&mut self, slot_index: SlotIndex) -> io::Result<()> {
		let slot_diary_pos = self.root.slot_diary_pos(slot_index);
		if let Some(pos) = slot_diary_pos {
			self.slot_reader.seek(pos)?;
		} else {
			self.slot_reader.unseek();
		}
		Ok(())
	}
	pub fn read(&self) -> io::Result<&Option<Slot>> { self.slot_reader.read() }
	pub fn new(root: Root, diary_reader: &'a mut diary::Reader) -> io::Result<Self> {
		let slot_reader = slot::Reader::new(diary_reader);
		Ok(Reader { root, slot_reader })
	}
}

impl Root {
	pub fn slot_diary_pos(&self, slot_index: SlotIndex) -> Option<diary::Pos> {
		let slot_mask = slot_index.as_mask();
		if self.mask & slot_mask > 0 {
			let predecessor_count = self.count_predecessors(slot_mask);
			let slot_offset = predecessor_count * slot::SLOT_LEN;
			let slot_pos = self.pos as usize + slot_offset;
			Some(diary::Pos::at(slot_pos))
		} else {
			None
		}
	}

	fn count_predecessors(&self, slot_mask: u32) -> usize {
		let predecessor_mask = slot_mask - 1;
		let predecessor_map = self.mask & predecessor_mask;
		predecessor_map.count_ones() as usize
	}
}