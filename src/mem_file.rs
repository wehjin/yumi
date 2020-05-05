use std::{io, thread};
use std::io::{Cursor, ErrorKind, Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};

use crate::util;
use crate::util::io_error;

#[derive(Debug, Clone)]
pub struct MemFile {
	tx: SyncSender<MemFileAction>
}

enum MemFileAction {
	Read(Sender<io::Result<Entry>>),
	Seek(usize, Sender<io::Result<()>>),
	Write(Entry, Sender<io::Result<(usize, u64)>>),
	Length(Sender<io::Result<usize>>),
}

impl MemFile {
	pub fn new() -> Self {
		let (tx, rx) = sync_channel(64);
		thread::spawn(move || {
			let mut cursor: Cursor<Vec<u8>> = Cursor::new(Vec::new());
			let mut read_pos: u64 = 0;
			let mut write_pos: u64 = 0;
			for action in rx {
				match action {
					MemFileAction::Read(tx) => {
						tx.send(read(&mut cursor, &mut read_pos)).unwrap();
					}
					MemFileAction::Seek(pos, tx) => {
						read_pos = pos as u64;
						tx.send(Ok(())).unwrap();
					}
					MemFileAction::Write(entry, tx) => {
						tx.send(write(entry, &mut cursor, &mut write_pos)).unwrap()
					}
					MemFileAction::Length(tx) => {
						let len = write_pos as usize;
						tx.send(Ok(len)).unwrap();
					}
				}
			}
		});
		MemFile { tx }
	}
}

fn read(cursor: &mut Cursor<Vec<u8>>, read_pos: &mut u64) -> io::Result<Entry> {
	let mut buf = [0u8; 8];
	let pos = cursor.seek(SeekFrom::Start(*read_pos))?;
	cursor.read_exact(&mut buf)?;
	*read_pos = pos + 8;
	let flag = (buf[0] & 0x80) == 0x80;
	buf[0] &= 0x7f;
	let (a, b) = util::u32x2_of_buf(&buf);
	Ok(Entry { flag, a, b })
}

fn write(entry: Entry, cursor: &mut Cursor<Vec<u8>>, write_pos: &mut u64) -> io::Result<(usize, u64)> {
	let Entry { flag, a, b } = entry;
	if (a & 0x80000000) != 0 {
		Err(io::Error::new(ErrorKind::InvalidData, format!("High bit found in entry {:?}", entry)))?
	}
	let pos = cursor.seek(SeekFrom::Start(*write_pos))?;
	let mut buf = [0u8; 4];
	util::big_end_first_4(a, &mut buf);
	if flag {
		buf[0] |= 0x80;
	}
	cursor.write_all(&buf)?;
	util::big_end_first_4(b, &mut buf);
	cursor.write_all(&buf)?;
	*write_pos = pos + 8;
	Ok((8, *write_pos))
}

pub trait EntryFile {
	fn read_entry(&self) -> io::Result<Entry>;
	fn seek(&self, pos: usize) -> io::Result<()>;
	fn write_entry(&self, entry: Entry) -> io::Result<(usize, u64)>;
	fn len(&self) -> io::Result<usize>;
}

impl<T: Deref<Target=dyn EntryFile>> EntryFile for T {
	fn read_entry(&self) -> io::Result<Entry> { self.deref().read_entry() }
	fn seek(&self, pos: usize) -> io::Result<()> { self.deref().seek(pos) }
	fn write_entry(&self, entry: Entry) -> io::Result<(usize, u64)> { self.deref().write_entry(entry) }
	fn len(&self) -> io::Result<usize> { self.deref().len() }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Entry {
	pub flag: bool,
	pub a: u32,
	pub b: u32,
}

impl EntryFile for MemFile {
	fn read_entry(&self) -> io::Result<Entry> {
		let (tx, rx) = channel();
		self.tx.send(MemFileAction::Read(tx)).unwrap();
		rx.recv().map_err(io_error)?
	}

	fn seek(&self, pos: usize) -> io::Result<()> {
		let (tx, rx) = channel();
		self.tx.send(MemFileAction::Seek(pos, tx)).unwrap();
		rx.recv().map_err(io_error)?
	}

	fn write_entry(&self, entry: Entry) -> io::Result<(usize, u64)> {
		let (tx, rx) = channel();
		self.tx.send(MemFileAction::Write(entry, tx)).unwrap();
		rx.recv().map_err(io_error)?
	}

	fn len(&self) -> io::Result<usize> {
		let (tx, rx) = channel();
		self.tx.send(MemFileAction::Length(tx)).unwrap();
		rx.recv().map_err(io_error)?
	}
}
