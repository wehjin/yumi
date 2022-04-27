use recurvedb::{Target, Ring, Flight, Archer, Arrow};
use recurvedb::diary::{Diary, FlightPos};

#[test]
fn main() {
	let start_flight = Flight { archer: Archer::Unit, target: Target::Unit, ring: Ring::Unit, arrow: Some(Arrow::Number(3)) };
	let (path, pos) = {
		let diary = Diary::temp().unwrap();
		let mut writer = diary.writer().unwrap();
		let pos = writer.write_flight(&start_flight).unwrap();
		assert_eq!(pos, FlightPos { archer: 0.into(), target: 1.into(), ring: 2.into(), arrow: 3.into(), end: (4 + 8).into() });
		diary.commit(writer.end_size());
		let mut commit_reader = diary.reader().unwrap();
		let commit_flight = commit_reader.read_flight(pos).unwrap();
		assert_eq!(commit_flight, start_flight);
		(diary.file_path.to_owned(), pos)
	};
	let reload_diary = Diary::load(&path).unwrap();
	let mut reload_reader = reload_diary.reader().unwrap();
	let reload_flight = reload_reader.read_flight(pos).unwrap();
	assert_eq!(reload_flight, start_flight);
}
