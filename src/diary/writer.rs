use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom};

use crate::{diary, Say};
use crate::bytes::WriteBytes;
use crate::diary::{Pos, SayPos};

pub struct Writer {
	file: File,
	end_size: usize,
}

impl Writer {
	pub fn write(&mut self, value: &impl WriteBytes) -> io::Result<(diary::Pos, usize)> {
		let start = self.end_size;
		self.file.seek(SeekFrom::Start(start as u64))?;
		let value_pos: diary::Pos = start.into();
		let value_size = value.write_bytes(&mut self.file);
		match value_size {
			Ok(value_size) => {
				self.end_size = start + value_size;
				Ok((value_pos, value_size))
			}
			Err(e) => {
				self.file.set_len(start as u64)?;
				Err(e)
			}
		}
	}

	pub fn write_say(&mut self, say: &Say) -> io::Result<SayPos> {
		let start = self.end_size;
		match self.try_write(say) {
			Ok(pos) => Ok(pos),
			Err(e) => {
				self.file.set_len(start as u64)?;
				Err(e)
			}
		}
	}

	fn try_write(&mut self, say: &Say) -> io::Result<SayPos> {
		let start = self.end_size;
		let (sayer_start, sayer_size) = self.write(&say.sayer)?;
		let (subject_start, subject_size) = self.write(&say.subject)?;
		let (ship_start, ship_size) = self.write(&say.ship)?;
		let (said_start, said_size) = self.write(&say.said)?;
		let end = Pos::at(start + sayer_size + subject_size + ship_size + said_size);
		let say_pos = SayPos { sayer: sayer_start, subject: subject_start, ship: ship_start, said: said_start, end };
		Ok(say_pos)
	}

	pub fn end_size(&self) -> usize {
		self.end_size
	}

	pub fn new(file: File, file_len: usize) -> Writer {
		Writer { file, end_size: file_len }
	}
}
