use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::Say;
use crate::write_bytes::*;

#[cfg(test)]
mod tests {
	use crate::{Said, Say, Sayer, Ship, Subject};
	use crate::diary::{Diary, SayStarts};

	#[test]
	fn main() {
		let diary = Diary::temp();
		let mut writer = diary.writer().unwrap();
		let say = Say { sayer: Sayer::Unit, subject: Subject::Unit, ship: Ship::Unit, said: Some(Said::Number(3)) };
		let starts = writer.write(&say).unwrap();
		assert_eq!(starts, SayStarts { sayer_start: 0, subject_start: 1, ship_start: 2, said_start: 3 });
		assert_eq!(writer.pos, 4 + 8)
	}
}

pub struct Writer {
	pub file: File,
	pub pos: usize,
}

impl Writer {
	pub fn write(&mut self, say: &Say) -> io::Result<SayStarts> {
		let sayer_start = self.pos;
		let sayer_size = say.sayer.write_bytes(&mut self.file)?;
		let subject_start = sayer_start + sayer_size;
		let subject_size = say.subject.write_bytes(&mut self.file)?;
		let ship_start = subject_start + subject_size;
		let ship_size = say.ship.write_bytes(&mut self.file)?;
		let said_start = ship_start + ship_size;
		let said_size = say.said.write_bytes(&mut self.file)?;
		let say_end = said_start + said_size;
		self.pos = say_end;
		let starts = SayStarts { sayer_start, subject_start, ship_start, said_start };
		Ok(starts)
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SayStarts {
	pub sayer_start: usize,
	pub subject_start: usize,
	pub ship_start: usize,
	pub said_start: usize,
}

pub struct Diary {
	file_path: PathBuf,
}

impl Diary {
	pub fn writer(&self) -> io::Result<Writer> {
		let file = OpenOptions::new().append(true).create(true).open(&self.file_path)?;
		let pos = file.metadata()?.len() as usize;
		Ok(Writer { file, pos })
	}

	pub fn temp() -> Diary {
		let mut path = std::env::temp_dir();
		path.push(&format!("diary{}", rand::random::<u32>()));
		Diary::load(&path)
	}

	pub fn load(file_path: &Path) -> Diary {
		Diary { file_path: file_path.to_path_buf() }
	}
}
