#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct Song {
	pub melodies: Vec<Say>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Say {
	Assert(Sayer, Subject, Ship, Said),
	Retract(Sayer, Subject, Ship, Said),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Said {
	Number(u64)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Ship {
	Static(&'static str, &'static str)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Subject {
	Singer(Sayer),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Sayer {
	Named(String)
}
