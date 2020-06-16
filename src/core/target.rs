use std::io::{Read, Write};
use std::io;

use crate::{ObjectId, Target};
use crate::bytes::{ReadBytes, WriteBytes};

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::bytes::{ReadBytes, WriteBytes};
	use crate::Target;

	#[test]
	fn text() {
		let target = Target::String("Hello".into());
		let mut cursor = Cursor::new(Vec::new());
		target.write_bytes(&mut cursor).unwrap();
		cursor.set_position(0);
		let target_final = Target::read_bytes(&mut cursor).unwrap();
		assert_eq!(target_final, target);
	}
}

impl ReadBytes<Target> for Target {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			1 => {
				let n = u64::read_bytes(reader)?;
				Ok(Target::Number(n))
			}
			2 => {
				let s = String::read_bytes(reader)?;
				Ok(Target::String(s))
			}
			3 => {
				let object_id = ObjectId::read_bytes(reader)?;
				Ok(Target::Object(object_id))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Target {
	/// Returns the number of bytes written
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let bytes = match self {
			Target::Number(n) => {
				writer.write_all(&[1])?;
				n.write_bytes(writer)?
			}
			Target::String(s) => {
				writer.write_all(&[2])?;
				s.write_bytes(writer)?
			}
			Target::Object(object_id) => {
				writer.write_all(&[3])?;
				object_id.write_bytes(writer)?
			}
		};
		Ok(1 + bytes)
	}
}


