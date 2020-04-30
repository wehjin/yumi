use std::fmt;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::path::{Path, PathBuf};

pub use reader::*;

mod reader;

#[cfg(test)]
mod data;

#[cfg(test)]
mod tests {
	use std::env;
	use std::path::PathBuf;

	use crate::hamt::{data, Trie};

	#[test]
	fn first() {
		let t = Trie::new(&data::temp_path("hamt-trie-first"));
		println!("{:?}", t);
	}
}

struct Writer {
	file: File,
	tip: usize,
}

impl Writer {
	pub fn write(&mut self, key: &[u8], value: usize) {}
}

trait Key {}

trait Hasher<T> {
	fn hash(subject: &T, level: HashLevel) -> u32;
}

enum HashLevel {
	Start,
	Next(u32),
}

struct Trie {
	path: PathBuf
}

impl Trie {
	pub fn new(path: &Path) -> Self {
		Trie { path: path.to_owned() }
	}
}

impl Debug for Trie {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_fmt(format_args!("\nTrie: {:?}\n", self.path))
	}
}