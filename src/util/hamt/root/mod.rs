use std::io;
use std::io::{Read, Write};

use crate::util::bytes::{ReadBytes, WriteBytes};
use crate::util::hamt::slot::Slot;
pub(crate) use crate::util::hamt::slot::SLOT_LEN as ROOT_LEN;

pub(crate) use self::prod_ab::*;

mod prod_ab;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Root {
	pub pos: u32,
	pub mask: u32,
}

impl Root {
	pub const ZERO: Root = Root { pos: 0, mask: 0 };
}

impl WriteBytes for Root {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let slot = Slot::Root(self.to_owned());
		slot.write_bytes(writer)
	}
}

impl ReadBytes<Root> for Root {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		let slot = Slot::read_bytes(reader)?;
		let root = if let Slot::Root(root) = slot {
			root
		} else {
			Root::ZERO
		};
		Ok(root)
	}
}
