use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::{diary, ObjectId, Point, Say, Arrow};
use crate::bytes::ReadBytes;
use crate::Sayer;

pub struct Reader {
	file_path: PathBuf,
	pub file: File,
	pub file_size: usize,
}

impl Reader {
	pub fn read_say(&mut self, pos: diary::SayPos) -> io::Result<Say> {
		let sayer = self.read::<Sayer>(pos.sayer)?;
		let object = self.read::<ObjectId>(pos.object)?;
		let point = self.read::<Point>(pos.point)?;
		let arrow = self.read::<Arrow>(pos.arrow)?;
		let say = Say { sayer, object, point, arrow: Some(arrow) };
		Ok(say)
	}

	pub fn read<V: ReadBytes<V>>(&mut self, pos: diary::Pos) -> io::Result<V> {
		self.file.seek(SeekFrom::Start(pos.into()))?;
		V::read_bytes(&mut self.file)
	}

	pub fn new(file_path: &Path, file_size: usize) -> io::Result<Reader> {
		let file = OpenOptions::new().read(true).open(file_path)?;
		Ok(Reader { file, file_size, file_path: file_path.to_path_buf() })
	}
}

impl Clone for Reader {
	fn clone(&self) -> Self { Reader::new(&self.file_path, self.file_size).unwrap() }
}