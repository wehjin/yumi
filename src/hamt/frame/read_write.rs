use std::io;

use crate::diary;
use crate::hamt::{Root, slot};
use crate::hamt::frame::SlotIndex;
use crate::hamt::slot::Slot;

#[cfg(test)]
mod tests {
	use std::error::Error;

	use crate::diary::Diary;
	use crate::hamt::{frame, Root};
	use crate::hamt::frame::SlotIndex;
	use crate::hamt::slot::Slot;

// TODO Write multiple slots with optional prototype.

	#[test]
	fn write_slot() -> Result<(), Box<dyn Error>> {
		let slot = Slot::KeyValue(3, 30);
		let slot_index = SlotIndex::at(3);
		let (path, root) = {
			let diary = Diary::temp()?;
			let mut diary_writer = diary.writer()?;
			let mut writer = frame::Writer::new(&mut diary_writer);
			let root = writer.write_slot(slot, slot_index)?;
			(diary.file_path, root)
		};
		let diary = Diary::load(&path)?;
		let mut diary_reader = diary.reader()?;
		let mut reader = frame::Reader::new(root, &mut diary_reader)?;
		{
			reader.seek(slot_index)?;
			let reading = reader.read()?;
			assert_eq!(reading, &Some(slot));
		}
		{
			reader.seek(SlotIndex::at(0))?;
			let reading = reader.read()?;
			assert_eq!(reading, &None);
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
	pub fn write_slot(&mut self, slot: Slot, slot_index: SlotIndex) -> io::Result<Root> {
		let (pos, _size) = self.slot_writer.write(&slot)?;
		Ok(Root { pos: pos.u32(), mask: slot_index.as_mask() })
	}
	pub fn new(diary_writer: &'a mut diary::Writer) -> Self { Writer { slot_writer: slot::Writer::new(diary_writer) } }
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