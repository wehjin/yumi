use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::Path;

use crate::{diary, Said, Say, Ship, Subject};
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
		let ship = self.read::<Ship>(pos.ship)?;
		let said = self.read::<Option<Said>>(pos.said)?;
		let say = Say { sayer, subject, ship, said };
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
