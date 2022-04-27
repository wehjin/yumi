use std::error::Error;
use std::hash::{Hash, Hasher};

use echodb::diary::Diary;
use echodb::hamt::{Hamt, Key, Root};

#[cfg(test)]
mod diary;

#[test]
fn write_read() -> Result<(), Box<dyn Error>> {
	let key = TestKey { n: 5 };
	let diary = Diary::temp()?;
	let mut diary_writer = diary.writer()?;
	let mut hamt = Hamt::new(Root::ZERO);
	hamt.write_value(&key, &"Hello".to_string(), &mut diary_writer)?;

	let mut diary_reader = diary_writer.reader()?;
	let hamt_reader = hamt.reader()?;
	let value: Option<String> = hamt_reader.read_value(&key, &mut diary_reader)?;
	assert_eq!(value, Some("Hello".to_string()));
	Ok(())
}

#[test]
fn read_none_from_empty_diary() -> Result<(), Box<dyn Error>> {
	let key = TestKey { n: 5 };
	let diary = Diary::temp()?;
	let mut diary_reader = diary.reader()?;
	let hamt = Hamt::new(Root::ZERO);
	let reader = hamt.reader()?;
	let value: Option<String> = reader.read_value(&key, &mut diary_reader)?;
	assert_eq!(value, None);
	Ok(())
}

struct TestKey {
	n: u32,
}

impl Hash for TestKey {
	fn hash<H: Hasher>(&self, state: &mut H) { state.write_u32(self.n) }
}

impl Key for TestKey {}
