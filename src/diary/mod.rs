pub use self::diary::*;
pub use self::reader::*;
pub use self::writer::*;

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
			let pos = writer.write(&start_say).unwrap();
			assert_eq!(pos, SayPos { sayer_start: 0, subject_start: 1, ship_start: 2, said_start: 3, end: 4 + 8 });
			diary.commit(&writer);
			let mut commit_reader = diary.reader().unwrap();
			let commit_say = commit_reader.read(pos.sayer_start).unwrap();
			assert_eq!(commit_say, start_say);
			(diary.file_path.to_owned(), pos)
		};
		let reload_diary = Diary::load(&path).unwrap();
		let mut reload_reader = reload_diary.reader().unwrap();
		let reload_say = reload_reader.read(pos.sayer_start).unwrap();
		assert_eq!(reload_say, start_say);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SayPos {
	pub sayer_start: usize,
	pub subject_start: usize,
	pub ship_start: usize,
	pub said_start: usize,
	pub end: usize,
}

