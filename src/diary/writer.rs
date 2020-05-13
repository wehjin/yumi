use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::{diary, Say};
use crate::bytes::WriteBytes;
use crate::diary::{Pos, SayPos};

pub struct Writer {
	path: PathBuf,
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
		let (object_start, object_size) = self.write(&say.object)?;
		let (point_start, point_size) = self.write(&say.point)?;
		let target = match say.target {
			None => unimplemented!(),
			Some(it) => it
		};
		let (target_start, target_size) = self.write(&target)?;
		let end = Pos::at(start + sayer_size + object_size + point_size + target_size);
		let say_pos = SayPos { sayer: sayer_start, object: object_start, point: point_start, target: target_start, end };
		Ok(say_pos)
	}

	pub fn reader(&self) -> io::Result<diary::Reader> {
		diary::Reader::new(&self.path, self.end_size)
	}

	pub fn end_size(&self) -> usize { self.end_size }

	pub fn new(path: &Path, file_len: usize) -> io::Result<Writer> {
		let file = OpenOptions::new().append(true).create(true).open(path)?;
		file.set_len(file_len as u64)?;
		Ok(Writer { path: path.to_owned(), file, end_size: file_len })
	}
}
