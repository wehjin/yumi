use crate::hamt::frame::Frame;

#[cfg(test)]
mod tests {
	use crate::hamt::root::Root;

	#[test]
	fn write_read() {
		let root = Root::new();
	}
}

#[derive()]
pub(crate) enum MemRoot {}


#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Root {
	PosMask(u32, u32),
}

impl Root {
	pub fn new() -> Self { Root::PosMask(0, 0) }
}
