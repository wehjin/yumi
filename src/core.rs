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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sayer {
	Unit,
	Named(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Subject {
	Unit,
	Sayer(Sayer),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Ship {
	Unit,
	FieldGroup(String, String),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Said {
	Number(u64)
}

impl Key for Subject {}
