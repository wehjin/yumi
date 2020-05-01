use std::io::{Cursor, Read, Seek};
use std::io;
use std::rc::Rc;

use crate::hamt::frame::{Frame, Slot};

#[cfg(test)]
mod tests {
	use std::rc::Rc;

	use crate::hamt::data::{byte_cursor, test_hash};
	use crate::hamt::frame::Slot;
	use crate::hamt::reader::Reader;

	#[test]
	fn empty_produces_no_value() {
		let reader = Reader::new(byte_cursor(), 0, Rc::new(test_hash)).unwrap();
		let keys = [1u32, 2, 3, 4];
		keys.iter().for_each(|key| {
			let value = reader.read(*key);
			assert_eq!(value, None)
		});
	}

	#[test]
	fn empty_produces_empty_root() {
		let reader = Reader::new(byte_cursor(), 0, Rc::new(test_hash)).unwrap();
		let frame = reader.root_frame;
		frame.slots.iter().for_each(|slot| {
			assert_eq!(*slot, Slot::Empty)
		})
	}
}

pub(crate) struct Reader {
	source: Box<dyn Source>,
	root_pos: u64,
	pub root_frame: Frame,
	hasher: Rc<dyn Fn(u32, usize, u8) -> u8>,
}

impl Reader {
	pub fn read(&self, key: u32) -> Option<u32> {
		if self.root_pos == 0 {
			None
		} else {
			let index = (*self.hasher)(key, 0, 0);
			let frame = &self.root_frame;
			match &frame.slots[index as usize] {
				Slot::Empty => None,
				Slot::Value { key: slot_key, value } => {
					if *slot_key == key { Some(*value) } else { None }
				}
				Slot::Ref { .. } => unimplemented!(),
			}
		}
	}

	pub fn new(source: impl Source, root_pos: u64, hasher: Rc<dyn Fn(u32, usize, u8) -> u8 + 'static>) -> io::Result<Self> {
		let mut source: Box<dyn Source> = Box::new(source);
		let root_frame = if root_pos == 0 {
			Frame::empty()
		} else {
			Frame::read(&mut source, root_pos)?
		};
		Ok(Reader { source, root_pos, root_frame, hasher })
	}
}

pub trait Source: Seek + Read + 'static {}

impl Source for Cursor<Vec<u8>> {}

impl Source for Box<dyn Source> {}
