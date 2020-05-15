use std::io::{Read, Write};
use std::io;

use crate::bytes::{ReadBytes, WriteBytes};
use crate::hamt::Key;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ObjName {
	Unit,
	String(String),
}

impl ObjName {
	pub fn new<S: AsRef<str>>(s: S) -> Self {
		ObjName::String(s.as_ref().to_string())
	}
}

impl Key for ObjName {}

impl ReadBytes<ObjName> for ObjName {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(ObjName::Unit),
			1 => {
				let name = String::read_bytes(reader)?;
				Ok(ObjName::String(name))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for ObjName {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			ObjName::Unit => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			ObjName::String(name) => {
				writer.write_all(&[1])?;
				let name_len = name.write_bytes(writer)?;
				Ok(1 + name_len)
			}
		}
	}
}

