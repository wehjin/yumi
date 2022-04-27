use std::hash::Hash;

pub use arrow::*;
pub use object::*;
pub use point::*;

mod arrow;
mod object;
mod point;


#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Speech {
	pub says: Vec<Say>,
}

pub trait Writable {
	fn to_says(&self) -> Vec<Say>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Say {
	pub sayer: Sayer,
	pub object: ObjectId,
	pub point: Point,
	pub arrow: Option<Arrow>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sayer {
	Unit,
	Named(String),
}

