use std::error::Error;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

use crate::hamt::frame::Frame;
use crate::hamt::slot_indexer::SlotIndexer;
use crate::hamt::util;

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use crate::hamt::data::fixture::ZeroThenKeySlotIndexer;
	use crate::hamt::reader::{MemFile, Reader};
	use crate::hamt::slot::Slot;

	#[test]
	fn empty_produces_no_value() {
		let mem_file = MemFile::new();
		let reader = Reader::new(Arc::new(mem_file), 0, 0).unwrap();
		let keys = [1u32, 2, 3, 4];
		keys.to_vec().into_iter().for_each(|key| {
			let mut slot_indexer = ZeroThenKeySlotIndexer { key, transition_depth: 1 };
			let value = reader.read(&mut slot_indexer).unwrap();
			assert_eq!(value, None)
		});
	}

	#[test]
	fn empty_produces_empty_root() {
		let mem_file = MemFile::new();
		let reader = Reader::new(Arc::new(mem_file), 0, 0).unwrap();
		let frame = reader.root_frame;
		frame.slots.iter().for_each(|slot| {
			assert_eq!(*slot, Slot::Empty)
		})
	}
}

pub(crate) struct Reader {
	source: Arc<dyn EntryFile>,
	pub root_pos: usize,
	pub root_mask: u32,
	pub root_frame: Frame,
}

impl Reader {
	pub fn read(&self, slot_indexer: &mut impl SlotIndexer) -> Result<Option<u32>, Box<dyn Error>> {
		if self.root_pos == 0 {
			Ok(None)
		} else {
			let frame = &self.root_frame;
			self.read_indexer(slot_indexer, 0, &frame)
		}
	}

	pub fn read_indexer(&self, indexer: &mut impl SlotIndexer, depth: usize, frame: &Frame) -> Result<Option<u32>, Box<dyn Error>> {
		frame.read_indexer(indexer, depth, &self.source)
	}

	pub fn read_frame(&self, pos: usize, mask: u32) -> Result<Frame, Box<dyn Error>> {
		let frame = if pos == self.root_pos {
			self.root_frame.clone()
		} else if pos == 0 {
			assert_eq!(mask, 0);
			Frame::empty()
		} else {
			Frame::read(&self.source, pos, mask)?
		};
		Ok(frame)
	}

	pub fn new(source: Arc<dyn EntryFile>, root_pos: usize, root_mask: u32) -> Result<Self, Box<dyn Error>> {
		let frame_source = source.clone();
		let root_frame = if root_pos == 0 {
			Frame::empty()
		} else {
			Frame::read(&frame_source, root_pos, root_mask)?
		};
		Ok(Reader { source: source.clone(), root_pos, root_mask, root_frame })
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

#[derive(Debug, Clone)]
pub struct MemFile {
	tx: SyncSender<MemFileAction>
}

enum MemFileAction {
	Read(SyncSender<Entry>),
	Seek(usize, SyncSender<()>),
	Write(Entry, SyncSender<(usize, u64)>),
	Length(SyncSender<usize>),
}

impl EntryFile for MemFile {
	fn read_entry(&self) -> Result<Entry, Box<dyn Error>> {
		let (tx, rx) = sync_channel(1);
		self.tx.send(MemFileAction::Read(tx)).unwrap();
		let entry = rx.recv()?;
		Ok(entry)
	}

	fn seek(&self, pos: usize) -> Result<(), Box<dyn Error>> {
		let (tx, rx) = sync_channel(1);
		self.tx.send(MemFileAction::Seek(pos, tx)).unwrap();
		let out = rx.recv()?;
		Ok(out)
	}

	fn write_entry(&self, entry: Entry) -> Result<(usize, u64), Box<dyn Error>> {
		let (tx, rx) = sync_channel(1);
		self.tx.send(MemFileAction::Write(entry, tx)).unwrap();
		let result = rx.recv()?;
		Ok(result)
	}

	fn len(&self) -> Result<usize, Box<dyn Error>> {
		let (tx, rx) = sync_channel(1);
		self.tx.send(MemFileAction::Length(tx)).unwrap();
		let pos = rx.recv()?;
		Ok(pos)
	}
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
