use std::io;

use crate::util::diary;
use crate::util::hamt::slot::Slot;

#[cfg(test)]
mod tests {
	use crate::util::diary::Diary;
	use crate::util::hamt::{Root, slot};
	use crate::util::hamt::slot::read_write::SLOT_LEN;
	use crate::util::hamt::slot::Slot;

	#[test]
	#[should_panic]
	fn empty() {
		assert_read_write(Slot::Empty);
	}

	#[test]
	fn key_value() {
		assert_read_write(Slot::KeyValue(3, 30));
	}

	#[test]
	fn root() {
		assert_read_write(Slot::Root(Root { pos: 24, mask: 0x00000003 }));
	}

	fn assert_read_write(slot: Slot) {
		let (file_path, pos) = {
			let diary = Diary::temp().unwrap();
			let mut diary_writer = diary.writer().unwrap();
			let mut writer = slot::Writer { diary_writer: &mut diary_writer };
			let (pos, size) = writer.write(&slot).unwrap();
			assert_eq!(size, SLOT_LEN);
			(diary.file_path, pos)
		};
		let diary = Diary::load(&file_path).unwrap();
		let mut diary_reader = diary.reader().unwrap();
		let mut reader = slot::Reader::new(&mut diary_reader);
		reader.seek(pos).unwrap();
		let reading = reader.read().unwrap();
		assert_eq!(reading, &Some(slot))
	}
}

pub(crate) struct Reader<'a> {
	diary_reader: &'a mut diary::Reader,
	value: Option<Slot>,
}

impl<'a> Reader<'a> {
	pub fn seek(&mut self, pos: diary::Pos) -> io::Result<diary::Pos> {
		let value = self.diary_reader.read(pos)?;
		self.value = Some(value);
		Ok(pos)
	}
	pub fn read(&self) -> io::Result<&Option<Slot>> { Ok(&self.value) }
	pub fn new(diary_reader: &'a mut diary::Reader) -> Self {
		Reader { diary_reader, value: None }
	}
}

pub(crate) struct Writer<'a> {
	diary_writer: &'a mut diary::Writer
}

impl<'a> Writer<'a> {
	pub fn write(&mut self, slot: &Slot) -> io::Result<(diary::Pos, usize)> {
		self.diary_writer.write(slot)
	}
	pub fn new(diary_writer: &'a mut diary::Writer) -> Self { Writer { diary_writer } }
}

pub(crate) static SLOT_LEN: usize = 8;

