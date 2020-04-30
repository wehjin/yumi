use std::time::Duration;

pub mod hamt;

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}


pub trait Verse<'a> {
	fn age() -> &'a Age;
}

pub struct Bang {
	_age: Age,
	_rays: Vec<Ray>,
}

pub struct Age {
	_since_epoch: Duration
}

pub struct Ray {
	_songs: Vec<Song>,
}

pub struct Song {
	_singer: PersonId,
	_subject: SubjectId,
	_object: ValueId,
	_ship: ShipId,
	_spin: Spin,
}

pub enum Spin {
	Up,
	Down,
}

pub enum ValueId {
	Thing(ThingId)
}

pub struct ShipId {
	_name: String,
	_space: String,
}

pub enum SubjectId {
	Person(PersonId),
	Place(PlaceId),
	Thing(ThingId),
}

pub enum PersonId {
	Name(String)
}

pub enum PlaceId {
	Url(String)
}

pub enum ThingId {
	Number(u64)
}
