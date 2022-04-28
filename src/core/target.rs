use std::io::{Read, Write};
use std::io;

use crate::util::bytes::{ReadBytes, WriteBytes};
use crate::util::hamt::Key;

/// A `Target` is location in the database where an `Arrow` can be attached.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Target {
	Unit,
	String(String),
}

impl Target {
	pub fn new<S: AsRef<str>>(s: S) -> Self { s.into() }
}

impl<T: AsRef<str>> From<T> for Target {
	fn from(s: T) -> Self {
		let s = s.as_ref().to_string();
		Target::String(s)
	}
}

impl Key for Target {}

impl ReadBytes<Target> for Target {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(Target::Unit),
			1 => {
				let name = String::read_bytes(reader)?;
				Ok(Target::String(name))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Target {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			Target::Unit => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Target::String(name) => {
				writer.write_all(&[1])?;
				let name_len = name.write_bytes(writer)?;
				Ok(1 + name_len)
			}
		}
	}
}

