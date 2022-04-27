use echodb::{Target, Ring, Say, Sayer, Arrow};
use echodb::diary::{Diary, SayPos};

#[test]
fn main() {
	let start_say = Say { sayer: Sayer::Unit, target: Target::Unit, ring: Ring::Unit, arrow: Some(Arrow::Number(3)) };
	let (path, pos) = {
		let diary = Diary::temp().unwrap();
		let mut writer = diary.writer().unwrap();
		let pos = writer.write_say(&start_say).unwrap();
		assert_eq!(pos, SayPos { sayer: 0.into(), target: 1.into(), ring: 2.into(), arrow: 3.into(), end: (4 + 8).into() });
		diary.commit(writer.end_size());
		let mut commit_reader = diary.reader().unwrap();
		let commit_say = commit_reader.read_say(pos).unwrap();
		assert_eq!(commit_say, start_say);
		(diary.file_path.to_owned(), pos)
	};
	let reload_diary = Diary::load(&path).unwrap();
	let mut reload_reader = reload_diary.reader().unwrap();
	let reload_say = reload_reader.read_say(pos).unwrap();
	assert_eq!(reload_say, start_say);
}
