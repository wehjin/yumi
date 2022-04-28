use std::cell::Cell;
use std::error::Error;
use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

use crate::util::diary::{Reader, Writer};
use crate::util;

/// `Diary` holds a file path and a virtual length for the file. Bytes
///  at locations higher that the virtual length are considered unreadable.
pub struct Diary {
	pub file_path: PathBuf,
	pub file_size: Cell<usize>,
}

impl Diary {
	/// Returns a `Diary` at the given `Path` using the file's file-system length
	/// as the diary length.
	pub fn load(file_path: &Path) -> io::Result<Diary> {
		let file_path = file_path.to_path_buf();
		let file_size = {
			let file = OpenOptions::new().write(true).create(true).open(&file_path)?;
			Cell::new(file.metadata()?.len() as usize)
		};
		Ok(Diary { file_path, file_size })
	}
	/// Opens a file reader at the diary's path using the diary's current length as the length of the file.
	pub fn reader(&self) -> io::Result<Reader> {
		Reader::new(&self.file_path, self.file_size.get())
	}

	/// Opens a file writer at the diary's path using the diary's current length as the starting
	/// for writing.  Only a single writer should be constructed.
	pub fn writer(&self) -> io::Result<Writer> {
		Writer::new(&self.file_path, self.file_size.get())
	}
	/// Allows a writer to change the diary's virtual length to include new bytes.
	pub fn commit(&self, size: usize) {
		self.file_size.set(size);
	}

	/// Creates a diary in the temp folder.
	pub fn temp() -> Result<Diary, Box<dyn Error>> {
		let mut path = util::temp_dir("diary")?;
		path.push("diary.dat");
		Ok(Diary::load(&path)?)
	}
}
