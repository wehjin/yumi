use std::cell::Cell;
use std::error::Error;
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

use crate::diary::{Reader, Writer};
use crate::util;

pub struct Diary {
	pub file_path: PathBuf,
	pub file_size: Cell<usize>,
}

impl Diary {
	pub fn reader(&self) -> io::Result<Reader> {
		Reader::new(&self.file_path, self.file_size.get())
	}
	pub fn commit(&self, size: usize) {
		self.file_size.set(size);
	}
	pub fn writer(&self) -> io::Result<Writer> {
		Writer::new(&self.file_path, self.file_size.get())
	}
	pub fn temp() -> Result<Diary, Box<dyn Error>> {
		let mut path = util::temp_dir("diary")?;
		path.push("diary.dat");
		let diary = Diary::load(&path)?;
		Ok(diary)
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
