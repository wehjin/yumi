use std::fmt::{Debug, Formatter};
use std::fmt;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests {
	use std::env;

	use crate::hamt::Trie;

	#[test]
	fn first() {
		let mut dir = env::temp_dir();
		dir.push("hamt-trie-first");
		println!("path: {:?}", dir);
		let t = Trie::new(&dir.as_path());
		println!("{:?}", t);
	}
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