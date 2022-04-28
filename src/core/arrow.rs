use std::io::{Read, Write};
use std::io;

use crate::Target;
use crate::util::bytes::{ReadBytes, WriteBytes};

/// An `Arrow` is  a piece of data stored in the database.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Arrow {
	Number(u64),
	String(String),
	Target(Target),
}

impl Arrow {
	pub fn as_target(&self) -> &Target {
		match self {
			Arrow::Target(target) => target,
			_ => panic!("Arrow is not a target")
		}
	}

	pub fn as_number(&self) -> u64 {
		match self {
			Arrow::Number(n) => *n,
			_ => panic!("Arrow is not a number")
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Arrow::String(s) => s,
			_ => panic!("Arrow is not text")
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			Arrow::Number(n) => format!("{}", n),
			Arrow::String(s) => s.to_string(),
			Arrow::Target(target) => format!("{:?}", target),
		}
	}
}

pub fn string_arrow_if_not_empty(s: impl AsRef<str>) -> Option<Arrow> {
	let s = s.as_ref();
	if s.is_empty() {
		None
	} else {
		Some(Arrow::String(s.to_string()))
	}
}

impl ReadBytes<Arrow> for Arrow {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			1 => {
				let n = u64::read_bytes(reader)?;
				Ok(Arrow::Number(n))
			}
			2 => {
				let s = String::read_bytes(reader)?;
				Ok(Arrow::String(s))
			}
			3 => {
				let target = Target::read_bytes(reader)?;
				Ok(Arrow::Target(target))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Arrow {
	/// Returns the number of bytes written
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let bytes = match self {
			Arrow::Number(n) => {
				writer.write_all(&[1])?;
				n.write_bytes(writer)?
			}
			Arrow::String(s) => {
				writer.write_all(&[2])?;
				s.write_bytes(writer)?
			}
			Arrow::Target(target) => {
				writer.write_all(&[3])?;
				target.write_bytes(writer)?
			}
		};
		Ok(1 + bytes)
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::Arrow;
	use crate::util::bytes::{ReadBytes, WriteBytes};

	#[test]
	fn text() {
		let arrow = Arrow::String("Hello".into());
		let mut cursor = Cursor::new(Vec::new());
		arrow.write_bytes(&mut cursor).unwrap();
		cursor.set_position(0);
		let arrow_final = Arrow::read_bytes(&mut cursor).unwrap();
		assert_eq!(arrow_final, arrow);
	}
}


