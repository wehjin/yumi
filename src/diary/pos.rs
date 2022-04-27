use std::fmt;
use std::ops::Add;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Pos {
	start: usize,
}

impl Pos {
	pub fn at(start: usize) -> Self { Pos { start } }
	pub fn u32(&self) -> u32 { self.start as u32 }
}

impl fmt::Display for Pos {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
	pub object: Pos,
	pub ring: Pos,
	pub arrow: Pos,
	pub end: Pos,
}
