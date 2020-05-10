use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::io;
use std::sync::Arc;

pub(crate) use root::*;

use crate::{diary, hamt};
use crate::bytes::{ReadBytes, WriteBytes};
use crate::diary::Diary;
use crate::hamt::hasher::UniversalHasher;
use crate::hamt::reader::{Reader, Reader2};
use crate::hamt::slot_indexer::{SlotIndexer, UniversalSlotPicker};
use crate::hamt::writer::{WriteContext, Writer, Writer2};
use crate::mem_file::MemFile;

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

	use crate::bytes::ReadBytes;
	use crate::diary::Diary;
	use crate::hamt::{Hamt2, Key, Root};

	#[test]
	fn write_read() -> Result<(), Box<dyn Error>> {
		let key = TestKey { n: 5 };
		let diary = Diary::temp()?;
		let mut diary_writer = diary.writer()?;
		let mut hamt = Hamt2::new(Root::ZERO);
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
		let hamt = Hamt2::new(Root::ZERO);
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

impl Reader2 {
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

pub(crate) struct Hamt2 {
	root: Root,
}

impl Hamt2 {
	pub fn write_value(&mut self, key: &impl hamt::Key, value: &impl WriteBytes, diary_writer: &mut diary::Writer) -> io::Result<()> {
		let key = key.universal(1);
		let mut slot_indexer = UniversalSlotPicker::new(key);
		let (pos, _size) = diary_writer.write(value)?;
		let mut writer = Writer2::new(self.root, diary_writer);
		self.root = writer.write(pos.u32(), &mut slot_indexer)?;
		Ok(())
	}
	pub fn reader(&self) -> io::Result<Reader2> { Ok(Reader2::new(self.root)) }
	pub fn new(root: Root) -> Self { Hamt2 { root } }
}

pub struct Hamt {
	value_diary: Diary,
	mem_file: MemFile,
	root: Root,
}

impl Hamt where {
	pub fn commit(&mut self, extender: Extender) {
		self.value_diary.commit(&extender.value_diary);
		self.root = extender.root;
	}
	pub fn extender(&self) -> Extender {
		Extender {
			value_diary: self.value_diary.writer().unwrap(),
			mem_file: self.mem_file.clone(),
			root: self.root.clone(),
		}
	}
	pub fn viewer<V: WriteBytes + ReadBytes<V>>(&self) -> Viewer<V> {
		Viewer {
			value_diary: self.value_diary.reader().unwrap(),
			value: None,
			mem_file: self.mem_file.clone(),
			root: self.root.clone(),
		}
	}
	pub fn new(values_diary: Diary) -> Hamt {
		Hamt {
			value_diary: values_diary,
			mem_file: MemFile::new(),
			root: Root::ZERO,
		}
	}
}

struct UniversalWriteScope {}

impl WriteContext for UniversalWriteScope {
	fn slot_indexer(&self, key: u32) -> Box<dyn SlotIndexer> {
		let universal_indexer = UniversalSlotPicker::new(key);
		Box::new(universal_indexer)
	}
}

pub struct Extender {
	value_diary: diary::Writer,
	mem_file: MemFile,
	root: Root,
}

impl Extender {
	pub fn extend<V: WriteBytes>(mut self, key: &impl Key, value: &V) -> io::Result<Self> {
		let (value_pos, _value_size) = self.value_diary.write(value)?;
		let mut writer = Writer::new(Arc::new(self.mem_file.clone()), self.root);
		let mut write_context = UniversalWriteScope {};
		let key = key.universal(1);
		writer.write(key, value_pos.into(), &mut write_context)?;
		Ok(Extender {
			value_diary: self.value_diary,
			mem_file: self.mem_file.clone(),
			root: writer.root(),
		})
	}
}

pub struct Viewer<V: ReadBytes<V>> {
	value_diary: diary::Reader,
	value: Option<V>,
	mem_file: MemFile,
	root: Root,
}

impl<V: ReadBytes<V>> Viewer<V> {
	pub fn value(&mut self, key: &impl Key) -> &Option<V> {
		let key = key.universal(1);
		let reader = Reader::new(Arc::new(self.mem_file.clone()), self.root).unwrap();
		let mut indexer = UniversalSlotPicker::new(key);
		let u32_pos = reader.read(&mut indexer).unwrap();
		let value = match u32_pos {
			None => None,
			Some(u32_pos) => {
				let value = self.value_diary.read::<V>((u32_pos as usize).into()).unwrap();
				Some(value)
			}
		};
		self.value = value;
		&self.value
	}
}

pub trait Key: Hash {
	fn universal(&self, level: u64) -> u32 {
		let mut hasher = UniversalHasher::new(level);
		self.hash(&mut hasher);
		(hasher.finish() as u32) & 0x7fffffff
	}
}
