use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};

use crate::{Say, Ship, Subject};
use crate::bytes::ReadBytes;
use crate::Sayer;

pub struct Reader {
	pub file: File,
	pub file_size: usize,
}

impl Reader {
	pub fn read(&mut self, sayer_start: usize) -> io::Result<Say> {
		self.file.seek(SeekFrom::Start(sayer_start as u64))?;
		let sayer = Sayer::read_bytes(&mut self.file)?;
		let subject = Subject::read_bytes(&mut self.file)?;
		let ship = Ship::read_bytes(&mut self.file)?;
		let said = Option::read_bytes(&mut self.file)?;
		let say = Say { sayer, subject, ship, said };
		Ok(say)
	}
}