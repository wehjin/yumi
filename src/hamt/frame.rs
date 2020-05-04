use std::io::{Error, SeekFrom};
use std::io;

use crate::hamt::reader::Source;
use crate::hamt::slot::Slot;
use crate::hamt::slot_indexer::SlotIndexer;
use crate::hamt::util;
use crate::hamt::util::u32_of_buf;
use crate::hamt::writer::Dest;

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::hamt::frame::Frame;
	use crate::hamt::writer::Dest;

	#[test]
	fn write_then_read() {
		let frame = Frame::clear().with_value_slot(0, 1, 7);
		let mut dest: Cursor<Vec<u8>> = Cursor::new(Vec::new());
		let (mask, pos) = frame.write(&mut dest).unwrap();
		assert_eq!(dest.position(), 8, "destination position");
		assert_eq!(pos, 8, "pos");
		assert_eq!(mask, 1 << 31, "mask");
		let (mut source, pos) = dest.as_source();
		let read = Frame::read(&mut source, pos as usize, mask).unwrap();
		assert_eq!(read, frame);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
		slots[index as usize] = Slot::PosMask(pos, mask);
		Frame { slots }
	}

	pub fn read_indexer(&self, indexer: &mut impl SlotIndexer, depth: usize, source: &mut impl Source) -> io::Result<Option<u32>> {
		let key = indexer.key();
		let index = indexer.slot_index(depth);
		let value = match self.slots[index as usize] {
			Slot::Empty => None,
			Slot::KeyValue(slot_key, value) => if slot_key == key {
				Some(value)
			} else {
				None
			},
			Slot::PosMask(pos, mask) => {
				let frame = Frame::read(source, pos as usize, mask)?;
				frame.read_indexer(indexer, depth + 1, source)?
			}
		};
		Ok(value)
	}

	pub fn read(source: &mut impl Source, pos: usize, mask: u32) -> io::Result<Frame> {
		let mut frame = Frame::clear();
		let mut next_seek = SeekFrom::Start((pos - 8) as u64);
		let mut mask = mask;
		let mut index: i8 = 31;
		while index >= 0 {
			let slot_present = mask & 1 == 1;
			if slot_present {
				source.seek(next_seek)?;
				next_seek = SeekFrom::Current(-16);
				frame.slots[index as usize] = Slot::read(source)?;
			}
			mask >>= 1;
			index -= 1;
		}
		Ok(frame)
	}

	fn read_mask(source: &mut impl Source, pos: usize) -> io::Result<(u32, usize)> {
		let mut mask_buf = [0u8; 4];
		source.seek(SeekFrom::Start((pos - mask_buf.len()) as u64))?;
		source.read_exact(&mut mask_buf)?;
		Ok((u32_of_buf(&mask_buf), mask_buf.len()))
	}

	pub fn write(&self, dest: &mut impl Dest) -> Result<(u32, usize), Error> {
		let (mask, _) = self.write_slots(dest)?;
		Ok((mask, dest.pos()))
	}

	fn write_mask(&self, mask: u32, dest: &mut impl Dest) -> io::Result<usize> {
		let mut mask_buf = [0u8; 4];
		util::big_end_first_4(mask, &mut mask_buf);
		dest.write_all(&mask_buf)?;
		Ok(4)
	}

	fn write_slots(&self, dest: &mut impl Dest) -> io::Result<(u32, usize)> {
		self.slots.iter().try_fold(
			(0u32, 0usize),
			|(mask, total_bytes), slot| {
				slot.write(dest).map(|bytes| {
					let mask_bit = if bytes == 0 { 0 } else { 1 };
					((mask << 1) | mask_bit, total_bytes + bytes)
				})
			},
		)
	}

	pub fn clear() -> Frame {
		let slots = [Slot::Empty; 32];
		Frame { slots }
	}
}
