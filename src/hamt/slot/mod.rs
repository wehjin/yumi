use std::io;
use std::io::{Read, Write};

use crate::bytes::{ReadBytes, WriteBytes};
use crate::hamt::root::Root;
use crate::util::{clr_high_bit, is_high_bit_set, set_high_bit, U32x2};

pub(crate) use self::read_write::{Reader, SLOT_LEN, Writer};

pub mod read_write;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Slot {
	Empty,
	KeyValue(u32, u32),
	Root(Root),
}

impl Default for Slot {
	fn default() -> Self { Slot::Empty }
}

impl WriteBytes for Slot {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let bytes = match self {
			Slot::Empty => panic!("write_bytes called on empty slot"),
			Slot::KeyValue(key, value) => {
				debug_assert!(!is_high_bit_set(*key));
				let key_bytes = key.write_bytes(writer)?;
				let val_bytes = value.write_bytes(writer)?;
				key_bytes + val_bytes
			}
			Slot::Root(root) => {
				debug_assert!(!is_high_bit_set(root.pos));
				let pos_bytes = set_high_bit(root.pos).write_bytes(writer)?;
				let mask_bytes = root.mask.write_bytes(writer)?;
				pos_bytes + mask_bytes
			}
		};
		assert_eq!(bytes, SLOT_LEN);
		Ok(bytes)
	}
}

impl ReadBytes<Slot> for Slot {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		let (a, b) = U32x2::read_bytes(reader)?;
		let slot = if is_high_bit_set(a) {
			let root = Root {
				pos: clr_high_bit(a),
				mask: b,
			};
			Slot::Root(root)
		} else {
			Slot::KeyValue(a, b)
		};
		Ok(slot)
	}
}
