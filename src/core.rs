use std::hash::Hash;

use crate::hamt::Key;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Speech {
	pub says: Vec<Say>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Say {
	pub sayer: Sayer,
	pub subject: Subject,
	pub ship: Ship,
	pub said: Option<Said>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Said {
	Number(u64)
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Ship {
	Unit,
	Static(&'static str, &'static str),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Subject {
	Unit,
	Sayer(Sayer),
}

impl Key for Subject {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sayer {
	Unit,
	Named(String),
}
