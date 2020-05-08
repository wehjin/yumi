#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) struct Root {
	pub pos: u32,
	pub mask: u32,
}

impl Root {
	pub const ZERO: Root = Root { pos: 0, mask: 0 };
}


