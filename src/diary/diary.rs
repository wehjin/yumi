use std::cell::Cell;
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

use crate::diary::{Reader, Writer};

pub struct Diary {
	pub file_path: PathBuf,
	pub file_size: Cell<usize>,
}

impl Diary {
	pub fn reader(&self) -> io::Result<Reader> {
		Reader::new(&self.file_path, self.file_size.get())
	}
	pub fn commit2(&self, size: usize) {
		self.file_size.set(size);
	}
	pub fn commit(&self, writer: &Writer) {
		let end_size = writer.end_size();
		self.commit2(end_size);
	}
	pub fn writer(&self) -> io::Result<Writer> {
		Writer::new(&self.file_path, self.file_size.get())
	}
	pub fn temp() -> io::Result<Diary> {
		let mut path = std::env::temp_dir();
		path.push(&format!("diary{}", rand::random::<u32>()));
		Diary::load(&path)
	}
	pub fn load(file_path: &Path) -> io::Result<Diary> {
		let file_path = file_path.to_path_buf();
		let file_size = {
			let file = OpenOptions::new().write(true).create(true).open(&file_path)?;
			Cell::new(file.metadata()?.len() as usize)
		};
		Ok(Diary { file_path, file_size })
	}
}
