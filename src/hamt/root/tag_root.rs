use std::io;
use std::io::{Read, Write};

use crate::bytes::{ReadBytes, WriteBytes};
use crate::hamt::Root;

#[cfg(test)]
mod tests {
	use std::error::Error;
	use std::io::{Cursor, Seek, SeekFrom};

	use crate::bytes::{ReadBytes, WriteBytes};
	use crate::hamt::{Root, TagRoot};

	#[test]
	fn write_read() -> Result<(), Box<dyn Error>> {
		let mut cursor = Cursor::new(Vec::new());
		let tag_root = TagRoot { tag: String::from("Alice"), root: Root::ZERO };
		tag_root.write_bytes(&mut cursor)?;
		cursor.seek(SeekFrom::Start(0))?;
		let tag_root_new = TagRoot::read_bytes(&mut cursor)?;
		assert_eq!(tag_root_new, tag_root);
		Ok(())
	}
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TagRoot<T>
	where T: WriteBytes + ReadBytes<T>
{
	tag: T,
	root: Root,
}

impl<T> WriteBytes for TagRoot<T>
	where T: WriteBytes + ReadBytes<T>
{
	fn write_bytes(&self, writer: &mut impl Write) -> io::Result<usize> {
		let tag_len = self.tag.write_bytes(writer)?;
		let root_len = self.root.write_bytes(writer)?;
		Ok(tag_len + root_len)
	}
}

impl<T> ReadBytes<TagRoot<T>> for TagRoot<T>
	where T: WriteBytes + ReadBytes<T>
{
	fn read_bytes(reader: &mut impl Read) -> io::Result<TagRoot<T>> {
		let tag = T::read_bytes(reader)?;
		let root = Root::read_bytes(reader)?;
		Ok(TagRoot { tag, root })
	}
}
