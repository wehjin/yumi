use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};

use crate::diary::SayPos;
use crate::Say;
use crate::write_bytes::WriteBytes;

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
