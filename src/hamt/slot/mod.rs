use crate::hamt::root::Root;

pub(crate) use self::read_write::{Reader, SLOT_LEN, Writer};

pub mod read_write;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Slot {
	Empty,
	KeyValue(u32, u32),
	Root(Root),
}

impl Default for Slot {
	fn default() -> Self { Slot::Empty }
}
