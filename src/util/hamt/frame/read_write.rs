use std::io;

use crate::util::diary;
use crate::util::hamt::{frame, Root, slot};
use crate::util::hamt::frame::SlotIndex;
use crate::util::hamt::slot::Slot;

#[cfg(test)]
mod tests {
	use std::error::Error;
	use std::path::Path;

	use crate::util::diary::Diary;
	use crate::util::hamt::{frame, Root};
	use crate::util::hamt::frame::read_write::WriteSlot;
	use crate::util::hamt::frame::SlotIndex;
	use crate::util::hamt::slot::Slot;

	#[test]
	fn revised_root() -> Result<(), Box<dyn Error>> {
		let slot_a = Slot::KeyValue(7, 70);
		let slot_b = Slot::KeyValue(7, 71);
		let slot_index = SlotIndex::at(7);
		let (path, root) = {
			let diary = Diary::temp()?;
			let mut diary_writer = diary.writer()?;
			let mut writer = frame::Writer::new(&mut diary_writer);
			// Write slot.
			let mut first_diary_reader = diary.reader()?;
			let mut first_reader = frame::Reader::new(Root::ZERO, &mut first_diary_reader)?;
			let first_root = writer.write_revised_root(WriteSlot { slot: slot_a, slot_index }, &mut first_reader)?;
			// Rewrite the slot.
			let mut second_diary_reader = diary.reader()?;
			let mut second_reader = frame::Reader::new(first_root, &mut second_diary_reader)?;
			let second_root = writer.write_revised_root(WriteSlot { slot: slot_b, slot_index }, &mut second_reader)?;
			(diary.file_path, second_root)
		};
		let mut slots = [Slot::Empty; 32];
		slots[slot_index.n as usize] = slot_b;
		assert_slots(&path, root, &slots)
	}

	#[test]
	fn sub_root_of_slots() -> Result<(), Box<dyn Error>> {
		let write_slot1 = WriteSlot { slot: Slot::KeyValue(1, 10), slot_index: SlotIndex::at(1) };
		let write_slot7 = WriteSlot { slot: Slot::KeyValue(7, 70), slot_index: SlotIndex::at(7) };
		let (path, root) = {
			let diary = Diary::temp()?;
			let mut diary_writer = diary.writer()?;
			let mut writer = frame::Writer::new(&mut diary_writer);
			let root = writer.write_root_with_slots(write_slot1, write_slot7)?;
			(diary.file_path, root)
		};
		let mut slots = [Slot::Empty; 32];
		slots[write_slot1.slot_index.n as usize] = write_slot1.slot;
		slots[write_slot7.slot_index.n as usize] = write_slot7.slot;
		assert_slots(&path, root, &slots)
	}

	#[test]
	fn sub_root_of_slot() -> Result<(), Box<dyn Error>> {
		let write_slot1 = WriteSlot { slot: Slot::KeyValue(1, 10), slot_index: SlotIndex::at(1) };
		let (path, root) = {
			let diary = Diary::temp()?;
			let mut diary_writer = diary.writer()?;
			let mut writer = frame::Writer::new(&mut diary_writer);
			let root = writer.write_root_with_slot(write_slot1)?;
			(diary.file_path, root)
		};
		let mut slots = [Slot::Empty; 32];
		slots[write_slot1.slot_index.n as usize] = write_slot1.slot;
		assert_slots(&path, root, &slots)
	}

	fn assert_slots(path: &Path, root: Root, slots: &[Slot; 32]) -> Result<(), Box<dyn Error>> {
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
		assert_eq!(reading, &Slot::Empty);
		Ok(())
	}
}

pub(crate) struct Writer<'a> {
	slot_writer: slot::Writer<'a>,
}

impl<'a> Writer<'a> {
	pub fn write_revised_root(&mut self, write_slot: WriteSlot, reader: &mut frame::Reader) -> io::Result<Root> {
		let mut first_pos: Option<diary::Pos> = None;
		let mut mask = 0u32;
		for n in SlotIndex::RANGE {
			let slot_index = SlotIndex::at(n);
			let slot = if write_slot.slot_index == slot_index {
				&write_slot.slot
			} else {
				reader.seek(slot_index)?;
				reader.read()?
			};
			if slot != &Slot::Empty {
				let (pos, _size) = self.slot_writer.write(slot)?;
				mask |= slot_index.as_mask();
				if first_pos.is_none() {
					first_pos = Some(pos)
				}
			}
		}
		let pos = first_pos.expect("No pos for first written slot");
		Ok(Root { pos: pos.u32(), mask })
	}
	pub fn write_root_with_slots(&mut self, write_slot_a: WriteSlot, write_slot_b: WriteSlot) -> io::Result<Root> {
		debug_assert_ne!(write_slot_a.slot_index, write_slot_b.slot_index);
		let (first_write, second_write) = if write_slot_a.slot_index < write_slot_b.slot_index {
			(write_slot_a, write_slot_b)
		} else {
			(write_slot_b, write_slot_a)
		};
		let (pos, _size) = self.slot_writer.write(&first_write.slot)?;
		self.slot_writer.write(&second_write.slot)?;
		Ok(Root { pos: pos.u32(), mask: first_write.slot_index.as_mask() | second_write.slot_index.as_mask() })
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
	slot: Slot,
}

impl<'a> Reader<'a> {
	pub fn seek(&mut self, slot_index: SlotIndex) -> io::Result<()> {
		let pos = self.root.slot_diary_pos(slot_index);
		self.slot = if let Some(pos) = pos {
			self.slot_reader.seek(pos)?;
			match self.slot_reader.read()? {
				None => Slot::Empty,
				Some(slot) => *slot,
			}
		} else {
			Slot::Empty
		};
		Ok(())
	}
	pub fn read(&self) -> io::Result<&Slot> { Ok(&self.slot) }
	pub fn new(root: Root, diary_reader: &'a mut diary::Reader) -> io::Result<Self> {
		let slot_reader = slot::Reader::new(diary_reader);
		Ok(Reader { root, slot_reader, slot: Slot::Empty })
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