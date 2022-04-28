use std::io;
use std::io::{Read, Write};

use crate::util::bytes::{ReadBytes, WriteBytes};

#[cfg(test)]
mod tests {
	use std::error::Error;
	use std::io::{Cursor, Seek, SeekFrom};

	use crate::util::bytes::{ReadBytes, WriteBytes};
	use crate::util::hamt::{ProdAB, Root};

	#[test]
	fn write_read() -> Result<(), Box<dyn Error>> {
		let mut cursor = Cursor::new(Vec::new());
		let tag_root = ProdAB { a: String::from("Alice"), b: Root::ZERO };
		tag_root.write_bytes(&mut cursor)?;
		cursor.seek(SeekFrom::Start(0))?;
		let tag_root_new = ProdAB::read_bytes(&mut cursor)?;
		assert_eq!(tag_root_new, tag_root);
		Ok(())
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ProdAB<T, U>
	where T: WriteBytes + ReadBytes<T>, U: WriteBytes + ReadBytes<U>
{
	pub a: T,
	pub b: U,
}

impl<T, U> WriteBytes for ProdAB<T, U>
	where T: WriteBytes + ReadBytes<T>, U: WriteBytes + ReadBytes<U>
{
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let a_len = self.a.write_bytes(writer)?;
		let b_len = self.b.write_bytes(writer)?;
		Ok(a_len + b_len)
	}
}

impl<T, U> ReadBytes<ProdAB<T, U>> for ProdAB<T, U>
	where T: WriteBytes + ReadBytes<T>, U: WriteBytes + ReadBytes<U>
{
	fn read_bytes(reader: &mut impl Read) -> io::Result<ProdAB<T, U>> {
		let a = T::read_bytes(reader)?;
		let b = U::read_bytes(reader)?;
		Ok(ProdAB { a, b })
	}
}
