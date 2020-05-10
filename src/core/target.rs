use std::io::{Read, Write};
use std::io;

use crate::bytes::{ReadBytes, WriteBytes};
use crate::Target;

impl ReadBytes<Option<Target>> for Option<Target> {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(None),
			1 => {
				let n = u64::read_bytes(reader)?;
				Ok(Some(Target::Number(n)))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Option<Target> {
	/// Returns the number of bytes written
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			None => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Some(target) => {
				match target {
					Target::Number(n) => {
						writer.write_all(&[1])?;
						let bytes = n.write_bytes(writer)?;
						Ok(1 + bytes)
					}
				}
			}
		}
	}
}


