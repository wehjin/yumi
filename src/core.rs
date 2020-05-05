use std::hash::Hash;

use crate::hamt::Key;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Speech {
	pub says: Vec<Say>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Say {
	Assert(Sayer, Subject, Ship, Said),
	Retract(Sayer, Subject, Ship, Said),
}

impl Say {
	pub fn said(&self) -> Option<&Said> {
		match self {
			Say::Assert(_sayer, _subject, _ship, said) => Some(said),
			Say::Retract(_sayer, _subject, _ship, _said) => None,
		}
	}
	pub fn subject(&self) -> &Subject {
		match self {
			Say::Assert(_sayer, subject, _ship, _) => subject,
			Say::Retract(_sayer, subject, _ship, _) => subject,
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Said {
	Number(u64)
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Ship {
	None,
	Static(&'static str, &'static str),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Subject {
	None,
	Sayer(Sayer),
}

impl Key for Subject {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Sayer {
	None,
	Named(String),
}
