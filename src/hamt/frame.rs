use std::error::Error;

use crate::hamt::root::Root;
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;
use crate::mem_file::EntryFile;

#[cfg(test)]
mod tests {
	use crate::hamt::frame::Frame;
	use crate::mem_file::{EntryFile, MemFile};

	#[test]
	fn write_then_read() {
		let frame = Frame::empty().with_value_slot(0, 1, 7);

		let dest = MemFile::new();
		let (mask, pos) = frame.write(&dest).unwrap();
		assert_eq!(dest.len().unwrap(), 8, "destination position");
		assert_eq!(pos, 8, "pos");
		assert_eq!(mask, 1 << 31, "mask");
		let (source, pos) = (dest.clone(), dest.len().unwrap());
		let read = Frame::read(&source, pos as usize, mask).unwrap();
		assert_eq!(read, frame);
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Frame {
	pub slots: [Slot; 32]
}

impl Frame {
	pub fn with_value_slot(&self, index: u8, key: u32, value: u32) -> Frame {
		let mut slots = self.slots.to_owned();
		slots[index as usize] = Slot::KeyValue(key, value);
		Frame { slots }
	}

	pub fn with_ref_slot(&self, index: u8, pos: u32, mask: u32) -> Frame {
		let mut slots = self.slots.to_owned();
		slots[index as usize] = Slot::Root(Root::PosMask(pos, mask));
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
				match root {
					Root::PosMask(pos, mask) => {
						// TODO Get the value directly instead of building a frame to get the value.
						let mask_val = *mask;
						let pos_val = *pos;
						let frame = Frame::read(source, pos_val as usize, mask_val)?;
						let value = frame.read_indexer(indexer, depth + 1, source)?;
						value
					}
					Root::Frame(frame) => {
						let value = frame.read_indexer(indexer, depth + 1, source)?;
						value
					}
				}
			}
		};
		Ok(value)
	}

	pub fn read(source: &impl EntryFile, pos: usize, mask: u32) -> Result<Frame, Box<dyn Error>> {
		let mut frame = Frame::empty();
		let mut next_seek = pos as i64 - 8;
		let mut mask = mask;
		let mut index: i8 = 31;
		while index >= 0 {
			let slot_present = mask & 1 == 1;
			if slot_present {
				source.seek(next_seek as usize)?;
				frame.slots[index as usize] = Slot::read(source)?;
				next_seek -= 8;
			}
			mask >>= 1;
			index -= 1;
		}
		Ok(frame)
	}

	pub fn write(&self, dest: &impl EntryFile) -> Result<(u32, usize), Box<dyn Error>> {
		let (mask, _) = self.write_slots(dest)?;
		let len = dest.len()?;
		Ok((mask, len))
	}

	fn write_slots(&self, dest: &impl EntryFile) -> Result<(u32, usize), Box<dyn Error>> {
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
