use std::hash::Hash;

pub use arrow::*;
pub use ring::*;
pub use target::*;

mod arrow;
mod target;
mod ring;


/// A `Volley` is a group of `Flights`.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Volley {
	pub flights: Vec<Flight>,
}

/// A `CanVolley` is anything that can become a `Volley`.
pub trait CanVolley {
	fn to_flights(&self) -> Vec<Flight>;
}

/// A `Flight` is an `Archer`, `Target`, `Ring`, and `Arrow` in one structure.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Flight {
	pub archer: Archer,
	pub target: Target,
	pub ring: Ring,
	pub arrow: Option<Arrow>,
}

impl CanVolley for Flight {
	fn to_flights(&self) -> Vec<Flight> { vec![self.clone()] }
}

/// An `Archer` is a writer of the database.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Archer {
	Unit,
	Named(String),
}

