use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};

pub use self::writer::*;

#[cfg(test)]
mod tests {
	use crate::{Said, Say, Sayer, Ship, Subject};
	use crate::diary::{Diary, SayPos};

	#[test]
	fn main() {
		let mut diary = Diary::temp().unwrap();
		let mut writer = diary.writer().unwrap();
		let say = Say { sayer: Sayer::Unit, subject: Subject::Unit, ship: Ship::Unit, said: Some(Said::Number(3)) };
		let starts = writer.write(&say).unwrap();
		assert_eq!(starts, SayPos { sayer_start: 0, subject_start: 1, ship_start: 2, said_start: 3, end: 4 + 8 });
		diary.commit(&writer);
		assert_eq!(diary.len_in_bytes(), 12)
	}
}

mod writer;

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
