use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::Say;
use crate::write_bytes::*;

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

pub struct Writer {
	pub file: File,
	pub file_size: usize,
}

impl Writer {
	pub fn write(&mut self, say: &Say) -> io::Result<SayPos> {
		let start = self.file_size;
		self.file.seek(SeekFrom::Start(start as u64))?;
		let say_pos = self.try_write(say, start);
		match say_pos {
			Ok(say_pos) => {
				self.file_size = say_pos.end;
				Ok(say_pos)
			}
			Err(e) => {
				let file_size = start as u64;
				self.file.set_len(file_size)?;
				Err(e)
			}
		}
	}

	fn try_write(&mut self, say: &Say, start: usize) -> io::Result<SayPos> {
		let sayer_start = start;
		let sayer_size = say.sayer.write_bytes(&mut self.file)?;
		let subject_start = sayer_start + sayer_size;
		let subject_size = say.subject.write_bytes(&mut self.file)?;
		let ship_start = subject_start + subject_size;
		let ship_size = say.ship.write_bytes(&mut self.file)?;
		let said_start = ship_start + ship_size;
		let said_size = say.said.write_bytes(&mut self.file)?;
		let end = said_start + said_size;
		let say_pos = SayPos { sayer_start, subject_start, ship_start, said_start, end };
		Ok(say_pos)
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
