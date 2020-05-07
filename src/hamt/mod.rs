use std::hash::{Hash, Hasher};
use std::io;
use std::sync::Arc;

pub(crate) use root::*;

use crate::bytes::{ReadBytes, WriteBytes};
use crate::diary;
use crate::diary::Diary;
use crate::hamt::hasher::UniversalHasher;
use crate::hamt::reader::Reader;
use crate::hamt::slot_indexer::{SlotIndexer, UniversalSlotPicker};
use crate::hamt::writer::{WriteContext, Writer};
use crate::mem_file::MemFile;

mod data;
mod frame;
mod hasher;
mod slot;
mod reader;
mod slot_indexer;
mod writer;
mod root;

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
			root: Root::new(),
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
		let Root::PosMask(root_pos, root_mask) = self.root;
		let mut writer = Writer::new(Arc::new(self.mem_file.clone()), root_pos as usize, root_mask);
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
		let Root::PosMask(root_pos, root_mask) = self.root;
		let reader = Reader::new(Arc::new(self.mem_file.clone()), root_pos as usize, root_mask).unwrap();
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
