use crate::hamt::frame::Frame;

#[cfg(test)]
mod tests {
	use crate::hamt::root::Root;

	#[test]
	fn write_read() {
		let root = Root::new();
	}
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Root {
	PosMask(u32, u32),
	Frame(Box<Frame>),
}

impl Root {
	pub fn new() -> Self {
		Root::Frame(Box::new(Frame::empty()))
	}
}
