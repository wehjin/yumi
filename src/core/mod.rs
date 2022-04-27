use std::hash::Hash;

pub use arrow::*;
pub use target::*;
pub use ring::*;

mod arrow;
mod target;
mod ring;


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
	pub target: Target,
	pub ring: Ring,
	pub arrow: Option<Arrow>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sayer {
	Unit,
	Named(String),
}

