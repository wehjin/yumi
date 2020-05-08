use std::error::Error;
use std::io;

use crate::hamt::root::Root;
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;
use crate::mem_file::EntryFile;

pub(crate) use self::read_write::{Reader, Writer};
pub(crate) use self::slot_index::SlotIndex;

mod read_write;
mod slot_index;

#[cfg(test)]
mod tests {
	use crate::hamt::frame::Frame;
	use crate::mem_file::{EntryFile, MemFile};

	#[test]
	fn write_then_read() {
		let frame = Frame::empty().with_value_slot(0, 1, 7);

		let dest = MemFile::new();
		let post_write_root = frame.write(&dest).unwrap();
		assert_eq!(post_write_root.pos, 8, "pos");
		assert_eq!(post_write_root.mask, 1 << 31, "mask");
		assert_eq!(dest.len().unwrap(), 8, "destination position");
		let source = dest.clone();
		let read = Frame::read(&source, post_write_root).unwrap();
		assert_eq!(read, frame);
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Frame {
	pub slots: [Slot; 32]
}

impl Frame {
	pub fn with_value_slot(&self, index: u8, key: u32, value: u32) -> Self {
		let mut slots = self.slots.to_owned();
		slots[index as usize] = Slot::KeyValue(key, value);
		Frame { slots }
	}

	pub fn with_ref_slot(&self, index: u8, root: Root) -> Self {
		let mut slots = self.slots.to_owned();
		slots[index as usize] = Slot::Root(root);
		Frame { slots }
	}

	pub fn read_indexer(&self, indexer: &mut impl SlotIndexer, depth: usize, source: &impl EntryFile) -> Result<Option<u32>, Box<dyn Error>> {
		let key = indexer.key();
		let index = indexer.slot_index(depth);
		let value = match &self.slots[index as usize] {
			Slot::Empty => None,
			Slot::KeyValue(slot_key, value) => {
				let slot_key_val = *slot_key;
				if slot_key_val == key {
					Some(*value)
				} else {
					None
				}
			}
			Slot::Root(root) => {
				// TODO Get the value directly instead of building a frame to get the value.
				let frame = Frame::read(source, *root)?;
				let value = frame.read_indexer(indexer, depth + 1, source)?;
				value
			}
		};
		Ok(value)
	}

	pub fn read(source: &impl EntryFile, root: Root) -> Result<Frame, Box<dyn Error>> {
		let mut frame = Frame::empty();
		let mut next_seek = root.pos as i64 - 8;
		let mut travelling_mask = root.mask;
		let mut travelling_index: i8 = 31;
		while travelling_index >= 0 {
			let slot_present = travelling_mask & 1 == 1;
			if slot_present {
				source.seek(next_seek as usize)?;
				frame.slots[travelling_index as usize] = Slot::read(source)?;
				next_seek -= 8;
			}
			travelling_mask >>= 1;
			travelling_index -= 1;
		}
		Ok(frame)
	}

	pub fn write(&self, dest: &impl EntryFile) -> io::Result<Root> {
		let (mask, _) = self.write_slots(dest)?;
		let len = dest.len()?;
		Ok(Root { pos: len as u32, mask })
	}

	fn write_slots(&self, dest: &impl EntryFile) -> io::Result<(u32, usize)> {
		self.slots.iter().try_fold(
			(0u32, 0usize),
			|(mask, total_bytes), slot| {
				slot.write(dest).map(|(bytes, _pos)| {
					let mask_bit = if bytes == 0 { 0 } else { 1 };
					((mask << 1) | mask_bit, total_bytes + bytes)
				})
			},
		)
	}

	pub fn empty() -> Frame {
		let slots = [
			Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty,
			Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty,
			Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty,
			Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty, Slot::Empty,
		];
		Frame { slots }
	}
}
