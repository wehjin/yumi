use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::{diary, Target, Ring, Flight, Arrow};
use crate::bytes::ReadBytes;
use crate::Archer;

pub struct Reader {
	file_path: PathBuf,
	pub file: File,
	pub file_size: usize,
}

impl Reader {
	pub fn read_flight(&mut self, pos: diary::FlightPos) -> io::Result<Flight> {
		let archer = self.read::<Archer>(pos.archer)?;
		let target = self.read::<Target>(pos.target)?;
		let ring = self.read::<Ring>(pos.ring)?;
		let arrow = self.read::<Arrow>(pos.arrow)?;
		let flight = Flight { archer, target, ring, arrow: Some(arrow) };
		Ok(flight)
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