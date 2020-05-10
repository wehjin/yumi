use std::io;

use crate::diary;
use crate::hamt::{frame, Root};
use crate::hamt::frame::SlotIndex;
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;

#[cfg(test)]
mod tests {
	use crate::diary::Diary;
	use crate::hamt::data::fixture::ZeroThenKeySlotIndexer;
	use crate::hamt::reader::Reader;
	use crate::hamt::Root;

	#[test]
	fn empty_produces_no_value() {
		let diary = Diary::temp().unwrap();
		let mut diary_reader = diary.reader().unwrap();
		let reader = Reader::new(Root::ZERO);
		for key in 1u32..4 {
			let mut slot_indexer = ZeroThenKeySlotIndexer { key, transition_depth: 0 };
			let value = reader.read(&mut slot_indexer, &mut diary_reader).unwrap();
			assert_eq!(value, None)
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Reader {
	root: Root,
}

impl Reader {
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
		Reader { root }
	}
}
