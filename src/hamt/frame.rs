use std::io::{Error, Read, SeekFrom, Write};
use std::io;

use crate::hamt::reader::Source;
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
		let write_frame = Frame::empty().update_value(0, 1, 7);
		let mut dest: Cursor<Vec<u8>> = Cursor::new(Vec::new());
		let bytes_written = write_frame.write(&mut dest).unwrap();
		assert_eq!(bytes_written, dest.position());

		let (mut source, pos) = dest.as_source();
		let read_frame = Frame::read(&mut source, pos).unwrap();
		assert_eq!(read_frame, write_frame);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Frame {
	pub slots: [Slot; 32]
}

impl Frame {
	pub fn empty() -> Frame {
		let slots = [Slot::Empty; 32];
		Frame { slots }
	}

	pub fn update_value(&self, index: u8, key: u32, value: u32) -> Frame {
		let mut slots = self.slots.to_owned();
		slots[index as usize] = Slot::Value { key, value };
		Frame { slots }
	}

	pub fn read(source: &mut impl Source, pos: u64) -> Result<Frame, Error> {
		let mut frame = Frame::empty();
		let mut mask = {
			let mut mask_buf = [0u8; 4];
			source.seek(SeekFrom::Start(pos - 4))?;
			source.read_exact(&mut mask_buf)?;
			u32_of_buf(&mask_buf)
		};
		let mut index: i8 = 31;
		let mut next_seek = SeekFrom::Current(-12);
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

	pub fn write(&self, dest: &mut impl Dest) -> Result<u64, Error> {
		let fold_result = self.slots.iter().try_fold((0u32, 0u64), |(mask, count), next| {
			next.write(dest)
				.map(|written| {
					let one_more = if written { 1u32 } else { 0u32 };
					((mask << 1) | one_more, count + one_more as u64)
				})
		});
		let (mask, slot_count) = fold_result?;
		let mut buf = [0u8; 4];
		util::big_end_first_4(mask, &mut buf);
		let bytes_written = ((slot_count * 2) + 1) * 4;
		dest.write(&buf).map(|_| bytes_written)
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Slot {
	Empty,
	Value { key: u32, value: u32 },
	Ref { pos: u64 },
}

impl Slot {
	pub fn write(&self, dest: &mut impl Write) -> io::Result<bool> {
		match self {
			Slot::Empty => Ok(false),
			Slot::Value { key, value } => {
				let mut buf = [0u8; 4];
				util::big_end_first_4(*key, &mut buf);
				dest.write(&buf)?;
				util::big_end_first_4(*value, &mut buf);
				dest.write(&buf).map(|_| true)
			}
			Slot::Ref { pos } => {
				let mut buf = [0u8; 8];
				big_end_first_8(*pos, &mut buf);
				buf[0] |= 0x80;
				dest.write(&buf).map(|_| true)
			}
		}
	}

	pub fn read(source: &mut impl Read) -> io::Result<Slot> {
		let mut buf = [0u8; 8];
		source.read_exact(&mut buf)?;
		let slot = if (buf[0] & 0x80) == 0 {
			let (key, value) = u32_pair_of_buf(&buf);
			Slot::Value { key, value }
		} else {
			buf[0] &= 0x7f;
			let pos = u64_of_buf(&buf);
			Slot::Ref { pos }
		};
		Ok(slot)
	}
}

fn big_end_first_8(n: u64, buf: &mut [u8; 8]) {
	buf[0] = (n >> 56) as u8;
	buf[1] = (n >> 48) as u8;
	buf[2] = (n >> 40) as u8;
	buf[3] = (n >> 32) as u8;
	buf[4] = (n >> 24) as u8;
	buf[5] = (n >> 16) as u8;
	buf[6] = (n >> 8) as u8;
	buf[7] = (n >> 0) as u8;
}

fn u32_pair_of_buf(buf: &[u8; 8]) -> (u32, u32) {
	(
		[
			(buf[0] as u32) << 24,
			(buf[1] as u32) << 16,
			(buf[2] as u32) << 8,
			(buf[3] as u32) << 0
		].iter().fold(0, |sum, next| sum | *next),
		[
			(buf[4] as u32) << 24,
			(buf[5] as u32) << 16,
			(buf[6] as u32) << 8,
			(buf[7] as u32) << 0
		].iter().fold(0, |sum, next| sum | *next)
	)
}


fn u64_of_buf(buf: &[u8; 8]) -> u64 {
	let values = [
		(buf[0] as u64) << 56,
		(buf[1] as u64) << 48,
		(buf[2] as u64) << 40,
		(buf[3] as u64) << 32,
		(buf[4] as u64) << 24,
		(buf[5] as u64) << 16,
		(buf[6] as u64) << 8,
		(buf[7] as u64) << 0
	];
	values.iter().fold(0, |sum, next| sum | *next)
}