use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};

use crate::Say;

#[cfg(test)]
mod tests {
	use crate::{Said, Say, Sayer, Ship, Subject};
	use crate::diary::Diary;

	#[test]
	fn main() {
		let diary = Diary::temp();
		let mut writer = diary.writer().unwrap();
		let say = Say::Assert(Sayer::None, Subject::None, Ship::None, Said::Number(3));
		let say_pos = writer.write_say(&say).unwrap();
		assert_eq!(say_pos, ());
	}
}

impl Writer {
	fn write_say(&mut self, say: &Say) -> io::Result<()> {
		Ok(())
	}
}

pub struct Writer {
	file: File,
	pos: u64,
}

impl Diary {
	pub fn writer(&self) -> io::Result<Writer> {
		let file = OpenOptions::new().append(true).create(true).open(&self.file_path)?;
		let pos = file.metadata()?.len();
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

pub struct Diary {
	file_path: PathBuf,
}
