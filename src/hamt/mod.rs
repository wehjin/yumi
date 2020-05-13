use std::hash::{Hash, Hasher};
use std::io;

pub(crate) use root::*;

use crate::{diary, hamt};
use crate::bytes::{ReadBytes, WriteBytes};
use crate::hamt::hasher::UniversalHasher;
use crate::hamt::slot_indexer::UniversalSlotPicker;
use crate::hamt::writer::Writer;

pub(crate) use self::reader::Reader;

pub(crate) mod frame;
mod data;
mod hasher;
mod slot;
mod reader;
mod slot_indexer;
mod writer;
mod root;

#[cfg(test)]
mod tests {
	use std::error::Error;
	use std::hash::{Hash, Hasher};

	use crate::diary::Diary;
	use crate::hamt::{Hamt, Key, Root};

	#[test]
	fn write_read() -> Result<(), Box<dyn Error>> {
		let key = TestKey { n: 5 };
		let diary = Diary::temp()?;
		let mut diary_writer = diary.writer()?;
		let mut hamt = Hamt::new(Root::ZERO);
		hamt.write_value(&key, &"Hello".to_string(), &mut diary_writer)?;

		let mut diary_reader = diary_writer.reader()?;
		let hamt_reader = hamt.reader()?;
		let value: Option<String> = hamt_reader.read_value(&key, &mut diary_reader)?;
		assert_eq!(value, Some("Hello".to_string()));
		Ok(())
	}

	#[test]
	fn read_none_from_empty_diary() -> Result<(), Box<dyn Error>> {
		let key = TestKey { n: 5 };
		let diary = Diary::temp()?;
		let mut diary_reader = diary.reader()?;
		let hamt = Hamt::new(Root::ZERO);
		let reader = hamt.reader()?;
		let value: Option<String> = reader.read_value(&key, &mut diary_reader)?;
		assert_eq!(value, None);
		Ok(())
	}

	struct TestKey { n: u32 }

	impl Hash for TestKey {
		fn hash<H: Hasher>(&self, state: &mut H) { state.write_u32(self.n) }
	}

	impl Key for TestKey {}
}

pub(crate) struct Hamt {
	pub root: Root,
}

impl Hamt {
	pub fn write_value(&mut self, key: &impl hamt::Key, value: &impl WriteBytes, diary_writer: &mut diary::Writer) -> io::Result<()> {
		let key = key.universal(1);
		let mut slot_indexer = UniversalSlotPicker::new(key);
		let (pos, _size) = diary_writer.write(value)?;
		let mut writer = Writer::new(self.root, diary_writer);
		self.root = writer.write(pos.u32(), &mut slot_indexer)?;
		Ok(())
	}
	pub fn reader(&self) -> io::Result<Reader> { Ok(Reader::new(self.root)) }
	pub fn new(root: Root) -> Self { Hamt { root } }
}

impl Reader {
	pub fn read_value<V: ReadBytes<V>>(&self, key: &impl hamt::Key, diary_reader: &mut diary::Reader) -> io::Result<Option<V>> {
		let key = key.universal(1);
		let mut slot_indexer = UniversalSlotPicker::new(key);
		let value = match self.read(&mut slot_indexer, diary_reader)? {
			None => None,
			Some(pos) => {
				let pos = diary::Pos::at(pos as usize);
				let value = diary_reader.read::<V>(pos)?;
				Some(value)
			}
		};
		Ok(value)
	}
}

pub trait Key: Hash {
	fn universal(&self, level: u64) -> u32 {
		let mut hasher = UniversalHasher::new(level);
		self.hash(&mut hasher);
		(hasher.finish() as u32) & 0x7fffffff
	}
}
