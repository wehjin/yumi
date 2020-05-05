use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub(crate) use root::*;

use crate::hamt::hasher::UniversalHasher;

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
	index: HashMap<u32, usize>,
}

impl<V: Clone> Hamt<V> {
	pub fn commit(&mut self, extender: Extender<V>) {
		self.values = extender.values;
		self.index = extender.index;
	}
	pub fn extender(&self) -> Extender<V> {
		Extender { values: self.values.clone(), index: self.index.clone() }
	}
	pub fn viewer(&self) -> Viewer<V> {
		Viewer { values: self.values.clone(), index: self.index.clone() }
	}
	pub fn new() -> Hamt<V> {
		Hamt { values: Vec::new(), index: HashMap::new() }
	}
}

pub struct Extender<V> {
	values: Vec<V>,
	index: HashMap<u32, usize>,
}

impl<T: Clone> Extender<T> {
	pub fn extend(&self, key: &impl Key, value: &T) -> Self {
		let value_pos = self.values.len();
		let mut values = self.values.clone();
		values.push(value.to_owned());
		let mut index = self.index.clone();
		index.insert(key.universal(1), value_pos);
		Extender { values, index }
	}
}

pub struct Viewer<V> {
	values: Vec<V>,
	index: HashMap<u32, usize>,
}

impl<T> Viewer<T> {
	pub fn value(&self, key: &impl Key) -> Option<&T> {
		let key = key.universal(1);
		self.index.get(&key).and_then(|pos| self.values.get(*pos))
	}
}

pub trait Key: Hash {
	fn universal(&self, level: u64) -> u32 {
		let mut hasher = UniversalHasher::new(level);
		self.hash(&mut hasher);
		hasher.finish() as u32
	}
}
