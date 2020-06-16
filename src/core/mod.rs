use std::hash::Hash;

pub use object::*;
pub use point::*;
pub use target::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Speech {
	pub says: Vec<Say>
}

pub trait Writable {
	fn to_says(&self) -> Vec<Say>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Say {
	pub sayer: Sayer,
	pub object: ObjectId,
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Target {
	Number(u64),
	String(String),
	Object(ObjectId),
}

impl Target {
	pub fn as_object_id(&self) -> &ObjectId {
		match self {
			Target::Object(id) => id,
			_ => panic!("Target is not an object")
		}
	}

	pub fn as_number(&self) -> u64 {
		match self {
			Target::Number(n) => *n,
			_ => panic!("Target is not a number")
		}
	}

	pub fn as_str(&self) -> &str {
		match self {
			Target::String(s) => s,
			_ => panic!("Target is not text")
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			Target::Number(n) => format!("{}", n),
			Target::String(s) => s.to_string(),
			Target::Object(object_id) => format!("{:?}", object_id),
		}
	}
}

mod target;

