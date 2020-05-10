use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use crate::{diary, Point, Say, Subject, Target};
use crate::bytes::ReadBytes;
use crate::Sayer;

pub struct Reader {
	pub file: File,
	pub file_size: usize,
}

impl Reader {
	pub fn read_say(&mut self, pos: diary::SayPos) -> io::Result<Say> {
		let sayer = self.read::<Sayer>(pos.sayer)?;
		let subject = self.read::<Subject>(pos.subject)?;
		let point = self.read::<Point>(pos.point)?;
		let target = self.read::<Option<Target>>(pos.target)?;
		let say = Say { sayer, subject, point, target };
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
