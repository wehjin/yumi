pub use self::diary::Diary;
pub use self::pos::*;
pub use self::reader::Reader;
pub use self::writer::Writer;

mod writer;
mod reader;
mod diary;
mod pos;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SayPos {
	pub sayer: Pos,
	pub object: Pos,
	pub point: Pos,
	pub target: Pos,
	pub end: Pos,
}

