use std::io::{Read, Write};
use std::io;

use crate::bytes::{ReadBytes, WriteBytes};
use crate::ObjectId;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Arrow {
	Number(u64),
	String(String),
	Object(ObjectId),
}

impl Arrow {
	pub fn as_object_id(&self) -> &ObjectId {
		match self {
			Arrow::Object(id) => id,
			_ => panic!("Arrow is not an object")
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
			Arrow::Object(object_id) => format!("{:?}", object_id),
		}
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
				let object_id = ObjectId::read_bytes(reader)?;
				Ok(Arrow::Object(object_id))
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
			Arrow::Object(object_id) => {
				writer.write_all(&[3])?;
				object_id.write_bytes(writer)?
			}
		};
		Ok(1 + bytes)
	}
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::Arrow;
	use crate::bytes::{ReadBytes, WriteBytes};

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


