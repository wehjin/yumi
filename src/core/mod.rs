use std::hash::Hash;

pub use point::*;
pub use target::*;

use crate::hamt::Key;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Speech {
	pub says: Vec<Say>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Say {
	pub sayer: Sayer,
	pub object: Object,
	pub point: Point,
	pub target: Option<Target>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sayer {
	Unit,
	Named(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Object {
	Unit,
	Sayer(Sayer),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Point {
	Unit,
	NameAspect(String, String),
}

mod point;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Target {
	Number(u64)
}

mod target;

impl Key for Object {}
