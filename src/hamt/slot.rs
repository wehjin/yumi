use std::error::Error;
use std::io;
use std::io::ErrorKind;

use crate::hamt::root::Root;
use crate::mem_file::{Entry, EntryFile};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Slot {
	Empty,
	KeyValue(u32, u32),
	Root(Root),
}

impl Default for Slot {
	fn default() -> Self { Slot::Empty }
}

impl Slot {
	pub fn read(source: &impl EntryFile) -> Result<Slot, Box<dyn Error>> {
		let Entry { flag, a, b } = source.read_entry()?;
		let slot = if flag {
			Slot::KeyValue(a, b)
		} else {
			Slot::Root(Root { pos: a, mask: b })
		};
		Ok(slot)
	}

	pub fn write(&self, dest: &impl EntryFile) -> io::Result<(usize, Option<u64>)> {
		match self {
			Slot::Empty => Ok((0, None)),
			Slot::KeyValue(key, value) => {
				let a = *key;
				let b = *value;
				let (bytes, pos) = dest.write_entry(Entry { flag: true, a, b })
					.map_err(|it| io::Error::new(ErrorKind::Other, format!("KeyValue: {}", it.to_string())))?;
				Ok((bytes, Some(pos)))
			}
			Slot::Root(root) => {
				let a = root.pos;
				let b = root.mask;
				let (bytes, end_pos) = dest.write_entry(Entry { flag: false, a, b })
					.map_err(|it| io::Error::new(ErrorKind::Other, format!("Root: {}", it.to_string())))?;
				Ok((bytes, Some(end_pos)))
			}
		}
	}
}
