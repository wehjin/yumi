extern crate uuid;

use std::error::Error;
use std::io::ErrorKind;

use echodb::kvs;
use echodb::util::temp_dir;

#[test]
fn it_works() -> Result<(), Box<dyn Error>> {
	let equation = Equation {
		left: 2,
		right: "2".to_string(),
	};
	let difficulty = Difficulty::Easy;
	let kvs_name = "difficulties";
	let kvs_folder = temp_dir("kv-test")?;
	{
		let kvs = kvs::open(kvs_name, &kvs_folder)?;
		kvs.write(&equation, &difficulty)?;
	}
	let kvs = kvs::open(kvs_name, &kvs_folder)?;
	let catalog = kvs.catalog()?;
	let stored_difficulty = catalog.read(&equation, || Difficulty::Hard)?;
	assert_eq!(difficulty, stored_difficulty);
	Ok(())
}

#[derive(Debug, Hash)]
struct Equation {
	left: u64,
	right: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Difficulty {
	Hard,
	Easy,
}

impl kvs::Key for Equation {}

impl kvs::Value for Difficulty {
	fn to_value_string(&self) -> String {
		match self {
			Difficulty::Hard => "hard".to_string(),
			Difficulty::Easy => "easy".to_string(),
		}
	}

	fn from_value_string(s: &String) -> Result<Self, Box<dyn Error>> {
		match s.as_str() {
			"hard" => Ok(Difficulty::Hard),
			"easy" => Ok(Difficulty::Easy),
			_ => Err(std::io::Error::from(ErrorKind::InvalidData).into())
		}
	}
}
