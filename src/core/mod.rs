use std::hash::Hash;

pub use object::*;
pub use point::*;
pub use target::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Speech {
	pub says: Vec<Say>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Say {
	pub sayer: Sayer,
	pub object: ObjName,
	pub point: Point,
	pub target: Option<Target>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sayer {
	Unit,
	Named(String),
}

mod object;
mod point;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Target {
	Number(u64)
}

mod target;

