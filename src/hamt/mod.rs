use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::hamt::hasher::UniversalHasher;

mod data;
mod frame;
mod hasher;
mod slot;
mod reader;
mod slot_indexer;
mod util;
mod writer;
mod root;

pub struct Hamt<T> {
	values: HashMap<u32, T>
}

impl<T: Clone> Hamt<T> {
	pub fn commit(&mut self, extender: Extender<T>) {
		self.values = extender.values;
	}
	pub fn extender(&self) -> Extender<T> {
		Extender { values: self.values.clone() }
	}
	pub fn viewer(&self) -> Viewer<T> {
		Viewer { values: self.values.clone() }
	}
	pub fn new() -> Hamt<T> {
		Hamt { values: Default::default() }
	}
}

pub struct Extender<T> {
	values: HashMap<u32, T>
}

impl<T: Clone> Extender<T> {
	pub fn extend(&self, key: &impl Key, value: &T) -> Self {
		let mut values = self.values.clone();
		values.insert(key.universal(1), value.to_owned());
		Extender { values }
	}
}

pub struct Viewer<T> {
	values: HashMap<u32, T>
}

impl<T> Viewer<T> {
	pub fn value(&self, key: &impl Key) -> Option<&T> {
		self.values.get(&key.universal(1))
	}
}

pub trait Key: Hash {
	fn universal(&self, level: u64) -> u32 {
		let mut hasher = UniversalHasher::new(level);
		self.hash(&mut hasher);
		hasher.finish() as u32
	}
}

