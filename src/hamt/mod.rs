use std::hash::{Hash, Hasher};
use std::io;
use std::sync::Arc;

pub(crate) use root::*;

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

pub struct Hamt<V> {
	values: Vec<V>,
	mem_file: MemFile,
	root: Root,
}

impl<V: Clone> Hamt<V> {
	pub fn commit(&mut self, extender: Extender<V>) {
		self.values = extender.values;
		self.root = extender.root;
	}
	pub fn extender(&self) -> Extender<V> {
		Extender {
			values: self.values.clone(),
			mem_file: self.mem_file.clone(),
			root: self.root.clone(),
		}
	}
	pub fn viewer(&self) -> Viewer<V> {
		Viewer {
			values: self.values.clone(),
			mem_file: self.mem_file.clone(),
			root: self.root.clone(),
		}
	}
	pub fn new() -> Hamt<V> {
		Hamt {
			values: Vec::new(),
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

pub struct Extender<V> {
	values: Vec<V>,
	mem_file: MemFile,
	root: Root,
}

impl<T: Clone> Extender<T> {
	pub fn extend(&self, key: &impl Key, value: &T) -> io::Result<Self> {
		let key = key.universal(1);
		let value_pos = self.values.len();
		let mut values = self.values.clone();
		values.push(value.to_owned());
		let Root::PosMask(root_pos, root_mask) = self.root;
		let mut writer = Writer::new(Arc::new(self.mem_file.clone()), root_pos as usize, root_mask);
		let mut write_context = UniversalWriteScope {};
		writer.write(key, value_pos as u32, &mut write_context)?;
		Ok(Extender { values, mem_file: self.mem_file.clone(), root: writer.root() })
	}
}

pub struct Viewer<V> {
	values: Vec<V>,
	mem_file: MemFile,
	root: Root,
}

impl<T> Viewer<T> {
	pub fn value(&self, key: &impl Key) -> Option<&T> {
		let key = key.universal(1);
		let Root::PosMask(root_pos, root_mask) = self.root;
		let reader = Reader::new(Arc::new(self.mem_file.clone()), root_pos as usize, root_mask).unwrap();
		let mut indexer = UniversalSlotPicker::new(key);
		reader.read(&mut indexer).unwrap()
			.and_then(|pos| self.values.get(pos as usize))
	}
}

pub trait Key: Hash {
	fn universal(&self, level: u64) -> u32 {
		let mut hasher = UniversalHasher::new(level);
		self.hash(&mut hasher);
		(hasher.finish() as u32) & 0x7fffffff
	}
}
