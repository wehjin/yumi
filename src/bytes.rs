use std::io;
use std::io::{Read, Write};
use std::ops::Deref;

use crate::{Sayer, Subject};
use crate::util::{big_end_first_2, big_end_first_4, big_end_first_8, io_error_of_utf8, u16_of_buf, U32x2, u32x2_of_buf, u64_of_buf};

pub trait WriteBytes {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize>;
}

pub trait ReadBytes<T> {
	fn read_bytes(reader: &mut impl Read) -> io::Result<T>;
}

impl ReadBytes<Subject> for Subject {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(Subject::Unit),
			1 => {
				let sayer = Sayer::read_bytes(reader)?;
				Ok(Subject::Sayer(sayer))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Subject {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			Subject::Unit => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Subject::Sayer(sayer) => {
				writer.write_all(&[1])?;
				let sayer_size = sayer.write_bytes(writer)?;
				Ok(1 + sayer_size)
			}
		}
	}
}

impl ReadBytes<Sayer> for Sayer {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(Sayer::Unit),
			1 => Ok(Sayer::Named(String::read_bytes(reader)?)),
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Sayer {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			Sayer::Unit => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Sayer::Named(name) => {
				writer.write_all(&[1])?;
				let name_size = name.write_bytes(writer)?;
				Ok(1 + name_size)
			}
		}
	}
}

impl ReadBytes<String> for String {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		let byte_count = u16::read_bytes(reader)?;
		let mut bytes = vec![0u8; byte_count as usize];
		reader.read_exact(&mut bytes)?;
		let string = String::from_utf8(bytes).map_err(io_error_of_utf8)?;
		Ok(string)
	}
}

impl WriteBytes for String {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		self.deref().write_bytes(writer)
	}
}

impl WriteBytes for str {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let str_bytes = self.as_bytes();
		let length_bytes = &mut [0u8; 2];
		{
			big_end_first_2(str_bytes.len() as u16, length_bytes);
		}
		writer.write_all(length_bytes)?;
		writer.write_all(str_bytes)?;
		let size = length_bytes.len() + str_bytes.len();
		Ok(size)
	}
}

impl ReadBytes<u8> for u8 {
	fn read_bytes(reader: &mut impl Read) -> io::Result<u8> {
		let buf = &mut [0u8; 1];
		reader.read_exact(buf)?;
		Ok(buf[0])
	}
}

impl WriteBytes for u8 {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let bytes = [*self; 1];
		writer.write_all(&bytes)?;
		Ok(bytes.len())
	}
}

impl ReadBytes<u16> for u16 {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		let buf = &mut [0u8; 2];
		reader.read_exact(buf)?;
		Ok(u16_of_buf(buf))
	}
}

impl ReadBytes<U32x2> for U32x2 {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		let buf = &mut [0u8; 8];
		reader.read_exact(buf)?;
		Ok(u32x2_of_buf(buf))
	}
}

impl WriteBytes for u32 {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let mut bytes = [0u8; 4];
		big_end_first_4(*self, &mut bytes);
		writer.write_all(&bytes)?;
		Ok(bytes.len())
	}
}

impl WriteBytes for &u32 {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> { (*self).write_bytes(writer) }
}

impl ReadBytes<u64> for u64 {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		let buf = &mut [0u8; 8];
		reader.read_exact(buf)?;
		let n = u64_of_buf(buf);
		Ok(n)
	}
}

impl WriteBytes for u64 {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let mut bytes = [0u8; 8];
		big_end_first_8(*self, &mut bytes);
		writer.write_all(&bytes)?;
		Ok(bytes.len())
	}
}