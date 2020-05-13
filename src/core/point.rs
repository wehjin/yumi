use std::io::{Read, Write};
use std::io;

use crate::bytes::{ReadBytes, WriteBytes};

#[derive(Debug, Clone, Eq, Hash)]
pub enum Point {
	Unit,
	String { name: String, aspect: String },
	Static { name: &'static str, aspect: &'static str },
}

impl Default for Point {
	fn default() -> Self { Point::Unit }
}

impl<S: AsRef<str>> From<(S, S)> for Point {
	fn from((name, aspect): (S, S)) -> Self {
		Point::String {
			name: name.as_ref().to_owned(),
			aspect: aspect.as_ref().to_owned(),
		}
	}
}

impl ReadBytes<Point> for Point {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(Point::Unit),
			1 | 2 => {
				let name = String::read_bytes(reader)?;
				let aspect = String::read_bytes(reader)?;
				Ok(Point::String { name, aspect })
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Point {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			Point::Unit => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Point::String { name, aspect } => {
				writer.write_all(&[1])?;
				let name_len = name.write_bytes(writer)?;
				let aspect_len = aspect.write_bytes(writer)?;
				Ok(1 + name_len + aspect_len)
			}
			Point::Static { name, aspect } => {
				writer.write_all(&[2])?;
				let name_len = name.write_bytes(writer)?;
				let aspect_len = aspect.write_bytes(writer)?;
				Ok(1 + name_len + aspect_len)
			}
		}
	}
}

impl PartialEq for Point {
	fn eq(&self, other: &Self) -> bool {
		match self {
			Point::Unit => match other {
				Point::Unit => true,
				Point::String { .. } => false,
				Point::Static { .. } => false
			}
			Point::String { name: name_a, aspect: aspect_a } => match other {
				Point::Unit => false,
				Point::String { name, aspect } => name_a == name && aspect_a == aspect,
				Point::Static { name, aspect } => name_a == name && aspect_a == aspect,
			},
			Point::Static { name: name_a, aspect: aspect_a } => match other {
				Point::Unit => false,
				Point::String { name, aspect } => name_a == name && aspect_a == aspect,
				Point::Static { name, aspect } => name_a == name && aspect_a == aspect,
			}
		}
	}
}
