use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use crate::{diary, Object, Point, Say, Target};
use crate::bytes::ReadBytes;
use crate::Sayer;

pub struct Reader {
	pub file: File,
	pub file_size: usize,
}

impl Reader {
	pub fn read_say(&mut self, pos: diary::SayPos) -> io::Result<Say> {
		let sayer = self.read::<Sayer>(pos.sayer)?;
		let object = self.read::<Object>(pos.object)?;
		let point = self.read::<Point>(pos.point)?;
		let target = self.read::<Target>(pos.target)?;
		let say = Say { sayer, object, point, target: Some(target) };
		Ok(say)
	}

	pub fn read<V: ReadBytes<V>>(&mut self, pos: diary::Pos) -> io::Result<V> {
		self.file.seek(SeekFrom::Start(pos.into()))?;
		V::read_bytes(&mut self.file)
	}

	pub fn new(file_path: &Path, file_size: usize) -> io::Result<Reader> {
		let file = OpenOptions::new().read(true).open(file_path)?;
		Ok(Reader { file, file_size })
	}
}
