#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Root {
	PosMask(u32, u32),
}

impl Root {
	pub fn new() -> Self { Root::PosMask(0, 0) }
}
