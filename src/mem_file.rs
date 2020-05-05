use std::error::Error;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use std::sync::mpsc::{channel, Sender, sync_channel, SyncSender};
use std::thread;

use crate::util;

#[derive(Debug, Clone)]
pub struct MemFile {
	tx: SyncSender<MemFileAction>
}

enum MemFileAction {
	Read(Sender<Entry>),
	Seek(usize, Sender<()>),
	Write(Entry, Sender<(usize, u64)>),
	Length(Sender<usize>),
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
						let mut buf = [0u8; 8];
						let pos = cursor.seek(SeekFrom::Start(read_pos)).unwrap();
						cursor.read_exact(&mut buf).unwrap();
						read_pos = pos + 8;
						let flag = (buf[0] & 0x80) == 0x80;
						buf[0] &= 0x7f;
						let (a, b) = util::u32x2_of_buf(&buf);
						let entry = Entry { flag, a, b };
						tx.send(entry).unwrap();
					}
					MemFileAction::Seek(pos, tx) => {
						read_pos = pos as u64;
						tx.send(()).unwrap();
					}
					MemFileAction::Write(Entry { flag, a, b }, tx) => {
						assert_eq!((a & 0x80), 0);
						let pos = cursor.seek(SeekFrom::Start(write_pos)).unwrap();
						let mut buf = [0u8; 4];
						util::big_end_first_4(a, &mut buf);
						if flag {
							buf[0] |= 0x80;
						}
						cursor.write_all(&buf).unwrap();
						util::big_end_first_4(b, &mut buf);
						cursor.write_all(&buf).unwrap();
						write_pos = pos + 8;
						tx.send((8, write_pos)).unwrap();
					}
					MemFileAction::Length(tx) => {
						let len = write_pos as usize;
						tx.send(len).unwrap();
					}
				}
			}
		});
		MemFile { tx }
	}
}

pub trait EntryFile {
	fn read_entry(&self) -> Result<Entry, Box<dyn Error>>;
	fn seek(&self, pos: usize) -> Result<(), Box<dyn Error>>;
	fn write_entry(&self, entry: Entry) -> Result<(usize, u64), Box<dyn Error>>;
	fn len(&self) -> Result<usize, Box<dyn Error>>;
}

impl<T: Deref<Target=dyn EntryFile>> EntryFile for T {
	fn read_entry(&self) -> Result<Entry, Box<dyn Error>> { self.deref().read_entry() }
	fn seek(&self, pos: usize) -> Result<(), Box<dyn Error>> { self.deref().seek(pos) }
	fn write_entry(&self, entry: Entry) -> Result<(usize, u64), Box<dyn Error>> { self.deref().write_entry(entry) }
	fn len(&self) -> Result<usize, Box<dyn Error>> { self.deref().len() }
}

pub struct Entry {
	pub flag: bool,
	pub a: u32,
	pub b: u32,
}

impl EntryFile for MemFile {
	fn read_entry(&self) -> Result<Entry, Box<dyn Error>> {
		let (tx, rx) = channel();
		self.tx.send(MemFileAction::Read(tx)).unwrap();
		let entry = rx.recv()?;
		Ok(entry)
	}

	fn seek(&self, pos: usize) -> Result<(), Box<dyn Error>> {
		let (tx, rx) = channel();
		self.tx.send(MemFileAction::Seek(pos, tx)).unwrap();
		let out = rx.recv()?;
		Ok(out)
	}

	fn write_entry(&self, entry: Entry) -> Result<(usize, u64), Box<dyn Error>> {
		let (tx, rx) = channel();
		self.tx.send(MemFileAction::Write(entry, tx)).unwrap();
		let result = rx.recv()?;
		Ok(result)
	}

	fn len(&self) -> Result<usize, Box<dyn Error>> {
		let (tx, rx) = channel();
		self.tx.send(MemFileAction::Length(tx)).unwrap();
		let pos = rx.recv()?;
		Ok(pos)
	}
}
