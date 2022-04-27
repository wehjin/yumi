use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::{diary, Flight};
use crate::bytes::WriteBytes;
use crate::diary::{Pos, FlightPos};

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

	pub fn write_flight(&mut self, flight: &Flight) -> io::Result<FlightPos> {
		let start = self.end_size;
		match self.try_write(flight) {
			Ok(pos) => Ok(pos),
			Err(e) => {
				self.file.set_len(start as u64)?;
				Err(e)
			}
		}
	}

	fn try_write(&mut self, flight: &Flight) -> io::Result<FlightPos> {
		let start = self.end_size;
		let (archer_start, archer_size) = self.write(&flight.archer)?;
		let (target_start, target_size) = self.write(&flight.target)?;
		let (ring_start, ring_size) = self.write(&flight.ring)?;
		let arrow = match &flight.arrow {
			None => unimplemented!(),
			Some(it) => it.clone(),
		};
		let (arrow_start, arrow_size) = self.write(&arrow)?;
		let end = Pos::at(start + archer_size + target_size + ring_size + arrow_size);
		let flight_pos = FlightPos { archer: archer_start, target: target_start, ring: ring_start, arrow: arrow_start, end };
		Ok(flight_pos)
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
