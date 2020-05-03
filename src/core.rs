pub(crate) struct Song {
	pub melodies: Vec<Melody>
}

pub enum Melody {
	Up(Singer, Subject, Ship, Say),
	Down(Singer, Subject, Ship, Say),
}

pub enum Spin {
	Up,
	Down,
}

pub enum Say {
	Number(u64)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Ship {
	Static(&'static str, &'static str)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Subject {
	Singer(Singer),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Singer {
	Named(String)
}

pub enum PlaceId {
	Url(String)
}
