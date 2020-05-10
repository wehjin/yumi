use std::io::{Read, Write};
use std::io;

use crate::bytes::{ReadBytes, WriteBytes};
use crate::Point;

impl Default for Point {
	fn default() -> Self { Point::Unit }
}

impl From<(&str, &str)> for Point {
	fn from((name, aspect): (&str, &str)) -> Self {
		Point::NameAspect(name.to_string(), aspect.to_string())
	}
}

impl ReadBytes<Point> for Point {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(Point::Unit),
			1 => {
				let field = String::read_bytes(reader)?;
				let group = String::read_bytes(reader)?;
				Ok(Point::NameAspect(field, group))
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
			Point::NameAspect(name, aspect) => {
				writer.write_all(&[1])?;
				let name_len = name.write_bytes(writer)?;
				let aspect_len = aspect.write_bytes(writer)?;
				Ok(1 + name_len + aspect_len)
			}
		}
	}
}
