use std::io::{Read, Write};
use std::io;

use crate::bytes::{ReadBytes, WriteBytes};
use crate::hamt::Key;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Object {
	Unit,
	String(String),
}

impl Key for Object {}

impl ReadBytes<Object> for Object {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(Object::Unit),
			1 => {
				let name = String::read_bytes(reader)?;
				Ok(Object::String(name))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Object {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			Object::Unit => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Object::String(name) => {
				writer.write_all(&[1])?;
				let name_len = name.write_bytes(writer)?;
				Ok(1 + name_len)
			}
		}
	}
}

