use std::io;
use std::io::Write;
use std::ops::Deref;

use crate::{Said, Sayer, Ship, Subject};
use crate::util::{big_end_first_2, big_end_first_8};

trait WriteBytes {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize>;
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
						let mut num_bytes = [0u8; 8];
						{
							big_end_first_8(*n, &mut num_bytes);
						}
						let num_bytes_size = num_bytes.len();
						writer.write_all(&num_bytes)?;
						let size = 1 + num_bytes_size;
						Ok(size)
					}
				}
			}
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
			Ship::Static(name, group) => {
				writer.write_all(&[1])?;
				let name_size = name.write_bytes(writer)?;
				let group_size = group.write_bytes(writer)?;
				let size = 1 + name_size + group_size;
				Ok(size)
			}
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

impl WriteBytes for String {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		self.deref().write_bytes(writer)
	}
}

impl WriteBytes for str {
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let str_bytes = self.as_bytes();
		let str_bytes_size = str_bytes.len();
		let length_bytes = &mut [0u8; 2];
		{
			big_end_first_2(str_bytes_size as u16, length_bytes);
		}
		let length_bytes_size = length_bytes.len();
		writer.write_all(length_bytes)?;
		writer.write_all(str_bytes)?;
		let size = length_bytes_size + str_bytes_size;
		Ok(size)
	}
}
