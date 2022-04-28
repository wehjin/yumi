use std::hash::{Hash, Hasher};
use std::io;

pub use root::*;

use crate::util::hamt;
use crate::util::bytes::{ReadBytes, WriteBytes};
use crate::util::hamt::frame::SlotIndex;
use crate::util::hamt::hasher::UniversalHasher;
use crate::util::hamt::slot::Slot;
use crate::util::hamt::slot_indexer::UniversalSlotPicker;
use crate::util::hamt::writer::Writer;
use crate::util::diary;

pub(crate) use self::reader::Reader;

pub(crate) mod frame;
mod root;
mod data;
mod hasher;
mod slot;
mod reader;
mod slot_indexer;
mod writer;

pub struct Hamt {
	pub root: Root,
}

impl Hamt {
	pub fn write_value(&mut self, key: &impl hamt::Key, value: &impl WriteBytes, diary_writer: &mut diary::Writer) -> io::Result<()> {
		let key = key.universal(1);
		let mut slot_indexer = UniversalSlotPicker::new(key);
		let (pos, _size) = diary_writer.write(value)?;
		let mut writer = Writer::new(self.root, diary_writer);
		self.root = writer.write(pos.u32(), &mut slot_indexer)?;
		Ok(())
	}
	pub fn reader(&self) -> io::Result<Reader> { Ok(Reader::new(self.root)) }
	pub fn new(root: Root) -> Self { Hamt { root } }
}

impl Reader {
	pub fn read_all<V: ReadBytes<V>>(&self, diary_reader: &mut diary::Reader) -> io::Result<Vec<V>> {
		let mut positions = Vec::new();
		{
			let mut roots = vec![self.root];
			loop {
				match roots.pop() {
					None => break,
					Some(root) => for n in SlotIndex::RANGE {
						match self.read_slot(root, SlotIndex::at(n), diary_reader)? {
							Slot::Empty => (),
							Slot::KeyValue(_, value) => positions.push(value),
							Slot::Root(root) => roots.push(root),
						}
					},
				}
			}
		}
		let values = positions.into_iter()
			.map(|it| diary_reader.read::<V>(diary::Pos::at(it as usize)))
			.collect();
		values
	}

	pub fn read_value<V: ReadBytes<V>>(&self, key: &impl hamt::Key, diary_reader: &mut diary::Reader) -> io::Result<Option<V>> {
		let key = key.universal(1);
		let mut slot_indexer = UniversalSlotPicker::new(key);
		let value = match self.read(&mut slot_indexer, diary_reader)? {
			None => None,
			Some(pos) => {
				let pos = diary::Pos::at(pos as usize);
				let value = diary_reader.read::<V>(pos)?;
				Some(value)
			}
		};
		Ok(value)
	}
}

pub trait Key: Hash {
	fn universal(&self, level: u64) -> u32 {
		let mut hasher = UniversalHasher::new(level);
		self.hash(&mut hasher);
		(hasher.finish() as u32) & 0x7fffffff
	}
}
