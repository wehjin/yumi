use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

pub use self::reader::*;
pub use self::writer::*;

mod writer;
mod reader;

#[cfg(test)]
mod tests {
	use crate::{Said, Say, Sayer, Ship, Subject};
	use crate::diary::{Diary, SayPos};

	#[test]
	fn main() {
		let start_say = Say { sayer: Sayer::Unit, subject: Subject::Unit, ship: Ship::Unit, said: Some(Said::Number(3)) };
		let (path, pos) = {
			let mut diary = Diary::temp().unwrap();
			let mut writer = diary.writer().unwrap();
			let pos = writer.write(&start_say).unwrap();
			assert_eq!(pos, SayPos { sayer_start: 0, subject_start: 1, ship_start: 2, said_start: 3, end: 4 + 8 });
			diary.commit(&writer);
			assert_eq!(diary.len_in_bytes(), 12);
			let mut commit_reader = diary.reader().unwrap();
			let commit_say = commit_reader.read(pos.sayer_start).unwrap();
			assert_eq!(commit_say, start_say);
			(diary.file_path.to_owned(), pos)
		};
		let reload_diary = Diary::load(&path).unwrap();
		let mut reload_reader = reload_diary.reader().unwrap();
		let reload_say = reload_reader.read(pos.sayer_start).unwrap();
		assert_eq!(reload_say, start_say);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SayPos {
	pub sayer_start: usize,
	pub subject_start: usize,
	pub ship_start: usize,
	pub said_start: usize,
	pub end: usize,
}

pub struct Diary {
	file_path: PathBuf,
	file_size: usize,
}

impl Diary {
	pub fn reader(&self) -> io::Result<Reader> {
		let file = OpenOptions::new().read(true).open(&self.file_path)?;
		let file_size = self.file_size;
		Ok(Reader { file, file_size })
	}

	pub fn commit(&mut self, writer: &Writer) {
		self.file_size = writer.file_size;
	}

	pub fn writer(&self) -> io::Result<Writer> {
		let file = OpenOptions::new().append(true).create(true).open(&self.file_path)?;
		let file_size = self.file_size;
		file.set_len(file_size as u64)?;
		Ok(Writer { file, file_size })
	}

	pub fn len_in_bytes(&self) -> usize { self.file_size }

	pub fn temp() -> io::Result<Diary> {
		let mut path = std::env::temp_dir();
		path.push(&format!("diary{}", rand::random::<u32>()));
		Diary::load(&path)
	}

	pub fn load(file_path: &Path) -> io::Result<Diary> {
		let file_path = file_path.to_path_buf();
		let file_size = {
			let file = OpenOptions::new().write(true).create(true).open(&file_path)?;
			file.metadata()?.len() as usize
		};
		Ok(Diary { file_path, file_size })
	}
}
