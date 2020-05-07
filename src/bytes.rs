use std::io;
use std::io::{Read, Write};
use std::ops::Deref;

use crate::{Said, Sayer, Ship, Subject};
use crate::util::{big_end_first_2, big_end_first_8, io_error_of_utf8, u16_of_buf, u64_of_buf};

pub trait WriteBytes {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize>;
}

pub trait ReadBytes<T> {
	fn read_bytes(reader: &mut impl Read) -> io::Result<T>;
}

impl ReadBytes<Option<Said>> for Option<Said> {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(None),
			1 => {
				let n = u64::read_bytes(reader)?;
				Ok(Some(Said::Number(n)))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Option<Said> {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			None => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Some(said) => {
				match said {
					Said::Number(n) => {
						writer.write_all(&[1])?;
						let bytes = n.write_bytes(writer)?;
						Ok(1 + bytes)
					}
				}
			}
		}
	}
}

impl ReadBytes<Ship> for Ship {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		match u8::read_bytes(reader)? {
			0 => Ok(Ship::Unit),
			1 => {
				let field = String::read_bytes(reader)?;
				let group = String::read_bytes(reader)?;
				Ok(Ship::FieldGroup(field, group))
			}
			_ => unimplemented!()
		}
	}
}

impl WriteBytes for Ship {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		match self {
			Ship::Unit => {
				writer.write_all(&[0])?;
				Ok(1)
			}
			Ship::FieldGroup(name, group) => {
				writer.write_all(&[1])?;
				let name_size = name.write_bytes(writer)?;
				let group_size = group.write_bytes(writer)?;
				Ok(1 + name_size + group_size)
			}
		}
	}
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
		let string_len = u16::read_bytes(reader)?;
		let mut bytes = Vec::with_capacity(string_len as usize);
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

impl ReadBytes<u16> for u16 {
	fn read_bytes(reader: &mut impl Read) -> io::Result<Self> {
		let buf = &mut [0u8; 2];
		reader.read_exact(buf)?;
		let n = u16_of_buf(buf);
		Ok(n)
	}
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