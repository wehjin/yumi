use std::io;
use std::io::{Read, Write};

use crate::bytes::{ReadBytes, WriteBytes};
use crate::diary;
use crate::hamt::Root;
use crate::hamt::slot::Slot;
use crate::util::{clr_high_bit, is_high_bit_set, set_high_bit};

#[cfg(test)]
mod tests {
	use crate::diary::Diary;
	use crate::hamt::{Root, slot};
	use crate::hamt::slot::read_write::SLOT_LEN;
	use crate::hamt::slot::Slot;

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
		let reading: &Option<Slot> = reader.read(pos).unwrap();
		assert_eq!(reading, &Some(slot))
	}
}

pub struct Reader<'a, V: ReadBytes<V>> {
	diary_reader: &'a mut diary::Reader,
	value: Option<V>,
}

impl<'a, V: ReadBytes<V>> Reader<'a, V> {
	fn read(&mut self, pos: diary::Pos) -> io::Result<&Option<V>> {
		let value = self.diary_reader.read(pos)?;
		self.value = Some(value);
		Ok(&self.value)
	}

	fn new(diary_reader: &'a mut diary::Reader) -> Self {
		Reader { diary_reader, value: None }
	}
}

impl ReadBytes<Slot> for Slot {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		let (a, b) = U32x2::read_bytes(reader)?;
		let slot = if is_high_bit_set(a) {
			let root = Root {
				pos: clr_high_bit(a),
				mask: b,
			};
			Slot::Root(root)
		} else {
			Slot::KeyValue(a, b)
		};
		Ok(slot)
	}
}

type U32x2 = (u32, u32);

pub struct Writer<'a> {
	diary_writer: &'a mut diary::Writer
}

impl<'a> Writer<'a> {
	fn write(&mut self, slot: &Slot) -> io::Result<(diary::Pos, usize)> {
		self.diary_writer.write(slot)
	}
}

impl WriteBytes for Slot {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let bytes = match self {
			Slot::Empty => panic!("write_bytes called on empty slot"),
			Slot::KeyValue(key, value) => {
				debug_assert!(!is_high_bit_set(*key));
				let key_bytes = key.write_bytes(writer)?;
				let val_bytes = value.write_bytes(writer)?;
				key_bytes + val_bytes
			}
			Slot::Root(root) => root.write_bytes(writer)?,
		};
		assert_eq!(bytes, SLOT_LEN);
		Ok(bytes)
	}
}

impl WriteBytes for Root {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		debug_assert!(!is_high_bit_set(self.pos));
		let pos_bytes = set_high_bit(self.pos).write_bytes(writer)?;
		let mask_bytes = self.mask.write_bytes(writer)?;
		Ok(pos_bytes + mask_bytes)
	}
}

static SLOT_LEN: usize = 8;

