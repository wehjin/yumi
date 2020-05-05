use std::error::Error;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;

use crate::hamt::frame::Frame;
use crate::hamt::slot_indexer::SlotIndexer;
use crate::mem_file::EntryFile;
use crate::util;

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use crate::hamt::data::fixture::ZeroThenKeySlotIndexer;
	use crate::hamt::reader::Reader;
	use crate::hamt::slot::Slot;
	use crate::mem_file::MemFile;

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

