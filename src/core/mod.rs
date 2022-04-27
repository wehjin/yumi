use std::hash::Hash;

pub use arrow::*;
pub use target::*;
pub use ring::*;

mod arrow;
mod target;
mod ring;


#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Speech {
	pub flights: Vec<Flight>,
}

pub trait Writable {
	fn to_flights(&self) -> Vec<Flight>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Flight {
	pub archer: Archer,
	pub target: Target,
	pub ring: Ring,
	pub arrow: Option<Arrow>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Archer {
	Unit,
	Named(String),
}

