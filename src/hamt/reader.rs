use std::error::Error;
use std::io;
use std::sync::Arc;

use crate::diary;
use crate::hamt::{frame, Root};
use crate::hamt::frame::{Frame, SlotIndex};
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;
use crate::mem_file::EntryFile;

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use crate::diary::Diary;
	use crate::hamt::data::fixture::ZeroThenKeySlotIndexer;
	use crate::hamt::reader::{Reader, Reader2};
	use crate::hamt::Root;
	use crate::hamt::slot::Slot;
	use crate::mem_file::MemFile;

	#[test]
	fn empty_produces_no_value() {
		let diary = Diary::temp().unwrap();
		let mut diary_reader = diary.reader().unwrap();
		let reader = Reader2::new(Root::ZERO);
		for key in 1u32..4 {
			let mut slot_indexer = ZeroThenKeySlotIndexer { key, transition_depth: 0 };
			let value = reader.read(&mut slot_indexer, &mut diary_reader).unwrap();
			assert_eq!(value, None)
		}
	}
}

pub(crate) struct Reader2 {
	root: Root,
}

impl Reader2 {
	pub fn read(&self, slot_indexer: &mut impl SlotIndexer, diary_reader: &mut diary::Reader) -> io::Result<Option<u32>> {
		let mut root = self.root;
		let mut depth = 0;
		let mut leaf_value = None;
		let mut done = false;
		while !done {
			let slot_index = SlotIndex::at(slot_indexer.slot_index(depth) as usize);
			match self.read_slot(root, slot_index, diary_reader)? {
				Slot::Root(sub_root) => {
					root = sub_root;
					depth += 1;
				}
				Slot::KeyValue(key, value) => {
					if key == slot_indexer.key() {
						leaf_value = Some(value);
					} else {
						leaf_value = None;
					}
					done = true;
				}
				Slot::Empty => {
					leaf_value = None;
					done = true;
				}
			}
		}
		Ok(leaf_value)
	}
	pub fn read_slot(&self, root: Root, slot_index: SlotIndex, diary_reader: &mut diary::Reader) -> io::Result<Slot> {
		debug_assert!(root.pos <= self.root.pos);
		let mut frame_reader = frame::Reader::new(root, diary_reader)?;
		frame_reader.seek(slot_index)?;
		let slot = frame_reader.read()?;
		Ok(*slot)
	}
	pub fn new(root: Root) -> Self {
		Reader2 { root }
	}
}

pub(crate) struct Reader {
	source: Arc<dyn EntryFile>,
	pub root: Root,
	pub root_frame: Frame,
}

impl Reader {
	pub fn read(&self, slot_indexer: &mut impl SlotIndexer) -> Result<Option<u32>, Box<dyn Error>> {
		if self.root == Root::ZERO {
			Ok(None)
		} else {
			let frame = &self.root_frame;
			self.read_indexer(slot_indexer, 0, &frame)
		}
	}

	pub fn read_indexer(&self, indexer: &mut impl SlotIndexer, depth: usize, frame: &Frame) -> Result<Option<u32>, Box<dyn Error>> {
		frame.read_indexer(indexer, depth, &self.source)
	}

	pub fn read_frame(&self, root: Root) -> Result<Frame, Box<dyn Error>> {
		let frame = if root == self.root {
			self.root_frame.clone()
		} else if root == Root::ZERO {
			Frame::empty()
		} else {
			Frame::read(&self.source, root)?
		};
		Ok(frame)
	}

	pub fn new(source: Arc<dyn EntryFile>, root: Root) -> Result<Self, Box<dyn Error>> {
		let frame_source = source.clone();
		let root_frame = if root == Root::ZERO {
			Frame::empty()
		} else {
			Frame::read(&frame_source, root)?
		};
		Ok(Reader { source: source.clone(), root, root_frame })
	}
}

