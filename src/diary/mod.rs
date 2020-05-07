use std::fmt;
use std::fmt::Formatter;
use std::ops::Add;

pub use self::diary::Diary;
pub use self::reader::Reader;
pub use self::writer::Writer;

mod writer;
mod reader;
mod diary;

#[cfg(test)]
mod tests {
	use crate::{Said, Say, Sayer, Ship, Subject};
	use crate::diary::{Diary, SayPos};

	#[test]
	fn main() {
		let start_say = Say { sayer: Sayer::Unit, subject: Subject::Unit, ship: Ship::Unit, said: Some(Said::Number(3)) };
		let (path, pos) = {
			let diary = Diary::temp().unwrap();
			let mut writer = diary.writer().unwrap();
			let pos = writer.write_say(&start_say).unwrap();
			assert_eq!(pos, SayPos { sayer: 0.into(), subject: 1.into(), ship: 2.into(), said: 3.into(), end: (4 + 8).into() });
			diary.commit(&writer);
			let mut commit_reader = diary.reader().unwrap();
			let commit_say = commit_reader.read_say(pos).unwrap();
			assert_eq!(commit_say, start_say);
			(diary.file_path.to_owned(), pos)
		};
		let reload_diary = Diary::load(&path).unwrap();
		let mut reload_reader = reload_diary.reader().unwrap();
		let reload_say = reload_reader.read_say(pos).unwrap();
		assert_eq!(reload_say, start_say);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Pos { start: usize }

impl Pos {
	pub fn at(start: usize) -> Self { Pos { start } }
}

impl fmt::Display for Pos {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str(&format!("{}", self.start))
	}
}

impl From<usize> for Pos {
	fn from(n: usize) -> Self { Pos { start: n } }
}

impl From<Pos> for usize {
	fn from(pos: Pos) -> Self { pos.start as Self }
}

impl From<Pos> for u64 {
	fn from(pos: Pos) -> Self { pos.start as Self }
}

impl From<Pos> for u32 {
	fn from(pos: Pos) -> Self { pos.start as Self }
}

impl Add<Pos> for Pos {
	type Output = Pos;
	fn add(self, rhs: Pos) -> Self::Output {
		Pos { start: self.start + rhs.start }
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SayPos {
	pub sayer: Pos,
	pub subject: Pos,
	pub ship: Pos,
	pub said: Pos,
	pub end: Pos,
}

