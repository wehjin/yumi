use std::io::{Read, Write};
use std::io;

use crate::bytes::{ReadBytes, WriteBytes};
use crate::hamt::Key;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ObjectId {
	Unit,
	String(String),
}

impl ObjectId {
	pub fn new<S: AsRef<str>>(s: S) -> Self {
		ObjectId::String(s.as_ref().to_string())
	}
}

impl Key for ObjectId {}

impl ReadBytes<ObjectId> for ObjectId {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(ObjectId::Unit),
			1 => {
				let name = String::read_bytes(reader)?;
				Ok(ObjectId::String(name))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for ObjectId {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			ObjectId::Unit => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			ObjectId::String(name) => {
				writer.write_all(&[1])?;
				let name_len = name.write_bytes(writer)?;
				Ok(1 + name_len)
			}
		}
	}
}

