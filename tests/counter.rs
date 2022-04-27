use std::{io, thread};
use std::error::Error;
use std::sync::mpsc::channel;

use echodb::{Echo, Object, ObjectFilter, ObjectId, Point, Say, Target, Writable};
use echodb::util::unique_name;

const COUNT: Point = Point::Static { name: "count", aspect: "Counter" };
const MAX_COUNT: Point = Point::Static { name: "max_count", aspect: "Counter" };

#[derive(Debug, Eq, PartialEq)]
struct Counter { object: Object }

impl Counter {
	pub fn count(&self) -> u64 {
		self.object[&COUNT].as_number()
	}

	pub fn new(name: &str, count: u64, max_count: u64) -> Self {
		let object = Object::new(
			&ObjectId::String(name.into()),
			vec![
				(&COUNT, Some(Target::Number(count))),
				(&MAX_COUNT, Some(Target::Number(max_count)))
			],
		);
		Counter { object }
	}
}

impl Writable for Counter {
	fn to_says(&self) -> Vec<Say> { self.object.to_says() }
}

impl<'a> ObjectFilter<'a> for Counter {
	fn key_point() -> &'a Point { &COUNT }
	fn data_points() -> &'a [&'a Point] { &[&COUNT, &MAX_COUNT] }
	fn from_name_and_properties(obj_name: &ObjectId, properties: Vec<(&Point, Option<Target>)>) -> Self {
		let object = Object::new(obj_name, properties);
		Counter { object }
	}
}

#[test]
fn filter() {
	let counter = Counter::new("card-counter", 7, 56);
	let mut chamber = {
		let echo = Echo::connect(&unique_name("point-holder"), &std::env::temp_dir());
		echo.write(|txn| txn.writable(&counter)).unwrap();
		echo.chamber().unwrap()
	};
	let counters = chamber.objects::<Counter>().unwrap();
	assert_eq!(1, counters.len());
	let final_counter = &counters[0];
	assert_eq!(final_counter, &counter);
	assert_eq!(final_counter.count(), 7)
}

#[test]
fn multi_thread() -> Result<(), Box<dyn Error>> {
	let echo = Echo::connect(&unique_name("test-multi-thread"), &std::env::temp_dir());
	let job1 = {
		let echo = echo.clone();
		thread::spawn(move || {
			echo.write(|write| {
				write.attributes(vec![(&COUNT, Target::Number(1))])
			})
		})
	};
	let job2 = {
		let (tx, rx) = channel::<Echo>();
		let job = thread::spawn(move || {
			for echo in rx {
				echo.write(|write| {
					write.attributes(vec![(&MAX_COUNT, Target::Number(100))])
				}).unwrap();
			}
			Ok(()) as io::Result<()>
		});
		tx.send(echo.clone()).unwrap();
		job
	};
	job1.join().unwrap()?;
	job2.join().unwrap()?;
	let chamber = echo.chamber()?;
	let attributes = chamber.properties(vec![&COUNT, &MAX_COUNT]);
	assert_eq!(attributes.len(), 2);
	Ok(())
}

#[test]
fn double_reconnect() -> Result<(), Box<dyn Error>> {
	let path = {
		let path = unique_name("echo-test-");
		let echo = Echo::connect(&path, &std::env::temp_dir());
		echo.write(|write| {
			write.target(Target::Number(3));
		})?;
		path
	};
	{
		let echo = Echo::connect(&path, &std::env::temp_dir());
		echo.write(|write| {
			write.target(Target::Number(10));
		})?;
	}
	let echo = Echo::connect(&path, &std::env::temp_dir());
	let mut chamber = echo.chamber()?;
	assert_eq!(chamber.target_or_none(), Some(Target::Number(10)));
	Ok(())
}

#[test]
fn reconnect() -> Result<(), Box<dyn Error>> {
	let path = {
		let path = unique_name("echo-test-");
		let echo = Echo::connect(&path, &std::env::temp_dir());
		echo.write(|write| {
			write.target(Target::Number(3));
			write.target(Target::Number(10));
		})?;
		path
	};
	let echo = Echo::connect(&path, &std::env::temp_dir());
	let mut chamber = echo.chamber()?;
	assert_eq!(chamber.target_or_none(), Some(Target::Number(10)));
	Ok(())
}

#[test]
fn objects_with_point() -> Result<(), Box<dyn Error>> {
	let dracula = ObjectId::new("Dracula");
	let bo_peep = ObjectId::new("Bo Peep");
	let echo = Echo::connect(&unique_name("echo-test-"), &std::env::temp_dir());
	echo.write(|shout| {
		shout.write_object_properties(&dracula, vec![(&COUNT, Target::Number(3)), ]);
		shout.write_object_properties(&bo_peep, vec![(&COUNT, Target::Number(7)), ]);
	})?;
	let mut objects = echo.chamber()?.objects_with_point(&COUNT)?;
	objects.sort();
	assert_eq!(objects, vec![bo_peep, dracula]);
	Ok(())
}

#[test]
fn object_attributes() -> Result<(), Box<dyn Error>> {
	let dracula = ObjectId::String("Dracula".into());
	let echo = Echo::connect(&unique_name("echo-test-"), &std::env::temp_dir());
	echo.write(|shout| {
		shout.write_object_properties(&dracula, vec![(&COUNT, Target::Number(3))]);
	})?;
	let attributes = echo.chamber()?.targets_at_object_points(&dracula, vec![&COUNT]);
	assert_eq!(attributes.get(&COUNT), Some(&Target::Number(3)));
	Ok(())
}

#[test]
fn attributes() -> Result<(), Box<dyn Error>> {
	let echo = Echo::connect(&unique_name("echo-test-"), &std::env::temp_dir());
	echo.write(|shout| {
		shout.attributes(vec![
			(&MAX_COUNT, Target::Number(100)),
			(&COUNT, Target::Number(0))
		]);
	})?;
	let attributes = echo.chamber()?.properties(vec![&MAX_COUNT, &COUNT]);
	assert_eq!(attributes, vec![
		(&MAX_COUNT, Some(Target::Number(100))),
		(&COUNT, Some(Target::Number(0)))
	]);
	Ok(())
}

#[test]
fn target() -> Result<(), Box<dyn Error>> {
	let echo = Echo::connect(&unique_name("echo-test-"), &std::env::temp_dir());
	let mut old_chamber = echo.chamber()?;
	echo.write(|write| {
		write.target(Target::Number(3))
	})?;
	let mut new_chamber = echo.chamber()?;
	assert_eq!(new_chamber.target_or_none(), Some(Target::Number(3)));
	assert_eq!(old_chamber.target_or_none(), None);
	Ok(())
}
