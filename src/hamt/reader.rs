use std::io::{Cursor, Read, Seek};
use std::io;

use crate::hamt::frame::{Frame, Slot};
use crate::hamt::slot_indexer::SlotIndexer;

#[cfg(test)]
mod tests {
	use crate::hamt::data::byte_cursor;
	use crate::hamt::data::fixture::ZeroThenKeySlotIndexer;
	use crate::hamt::frame::Slot;
	use crate::hamt::reader::Reader;

	#[test]
	fn empty_produces_no_value() {
		let reader = Reader::new(byte_cursor(), 0, 0).unwrap();
		let keys = [1u32, 2, 3, 4];
		keys.to_vec().into_iter().for_each(|key| {
			let mut slot_indexer = ZeroThenKeySlotIndexer { key };
			let value = reader.read(&mut slot_indexer);
			assert_eq!(value, None)
		});
	}

	#[test]
	fn empty_produces_empty_root() {
		let reader = Reader::new(byte_cursor(), 0, 0).unwrap();
		let frame = reader.root_frame;
		frame.slots.iter().for_each(|slot| {
			assert_eq!(*slot, Slot::Empty)
		})
	}
}

pub(crate) struct Reader {
	source: Box<dyn Source>,
	root_pos: usize,
	root_mask: u32,
	pub root_frame: Frame,
}

impl Reader {
	pub fn read(&self, slot_indexer: &mut impl SlotIndexer) -> Option<u32> {
		if self.root_pos == 0 {
			None
		} else {
			let key = slot_indexer.key();
			let slot_index = slot_indexer.slot_index(0);
			let frame = &self.root_frame;
			match &frame.slots[slot_index as usize] {
				Slot::Empty => None,
				Slot::Value { key: slot_key, value } => {
					if *slot_key == key {
						Some(*value)
					} else { None }
				}
				Slot::Ref { .. } => unimplemented!(),
			}
		}
	}

	pub fn new(source: impl Source, root_pos: usize, root_mask: u32) -> io::Result<Self> {
		let mut source: Box<dyn Source> = Box::new(source);
		let root_frame = if root_pos == 0 {
			Frame::clear()
		} else {
			Frame::read(&mut source, root_pos, root_mask)?
		};
		Ok(Reader { source, root_pos, root_mask, root_frame })
	}
}

pub trait Source: Seek + Read + 'static {}

impl Source for Cursor<Vec<u8>> {}

impl Source for Box<dyn Source> {}
