use std::io::{Read, Write};
use std::io;

use crate::util::bytes::{ReadBytes, WriteBytes};
use crate::util::hamt::Key;

/// A `Ring` is a sub-location on a `Target`.
#[derive(Debug, Clone, Eq, Hash)]
pub enum Ring {
	Center,
	String { aspect: String, name: String },
	Static { aspect: &'static str, name: &'static str },
}

impl Key for Ring {}

impl Default for Ring {
	fn default() -> Self { Ring::Center }
}

impl<S: AsRef<str>> From<(S, S)> for Ring {
	fn from((name, aspect): (S, S)) -> Self {
		Ring::String {
			name: name.as_ref().to_owned(),
			aspect: aspect.as_ref().to_owned(),
		}
	}
}

impl ReadBytes<Ring> for Ring {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(Ring::Center),
			1 | 2 => {
				let name = String::read_bytes(reader)?;
				let aspect = String::read_bytes(reader)?;
				Ok(Ring::String { name, aspect })
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Ring {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			Ring::Center => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Ring::String { name, aspect } => {
				writer.write_all(&[1])?;
				let name_len = name.write_bytes(writer)?;
				let aspect_len = aspect.write_bytes(writer)?;
				Ok(1 + name_len + aspect_len)
			}
			Ring::Static { name, aspect } => {
				writer.write_all(&[2])?;
				let name_len = name.write_bytes(writer)?;
				let aspect_len = aspect.write_bytes(writer)?;
				Ok(1 + name_len + aspect_len)
			}
		}
	}
}

impl PartialEq for Ring {
	fn eq(&self, other: &Self) -> bool {
		match self {
			Ring::Center => match other {
				Ring::Center => true,
				Ring::String { .. } => false,
				Ring::Static { .. } => false
			}
			Ring::String { name: name_a, aspect: aspect_a } => match other {
				Ring::Center => false,
				Ring::String { name, aspect } => name_a == name && aspect_a == aspect,
				Ring::Static { name, aspect } => name_a == name && aspect_a == aspect,
			},
			Ring::Static { name: name_a, aspect: aspect_a } => match other {
				Ring::Center => false,
				Ring::String { name, aspect } => name_a == name && aspect_a == aspect,
				Ring::Static { name, aspect } => name_a == name && aspect_a == aspect,
			}
		}
	}
}
