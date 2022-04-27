use std::{io, thread};
use std::error::Error;
use std::sync::mpsc::channel;

use echodb::{Arrow, Clout, CloutFilter, Echo, Ring, Flight, Target, Writable};
use echodb::util::unique_name;

const COUNT: Ring = Ring::Static { name: "count", aspect: "Counter" };
const MAX_COUNT: Ring = Ring::Static { name: "max_count", aspect: "Counter" };

#[derive(Debug, Eq, PartialEq)]
struct Counter {
	clout: Clout,
}

impl Counter {
	pub fn count(&self) -> u64 {
		self.clout[&COUNT].as_number()
	}

	pub fn new(name: &str, count: u64, max_count: u64) -> Self {
		let clout = Clout::new(
			&Target::String(name.into()),
			vec![
				(&COUNT, Some(Arrow::Number(count))),
				(&MAX_COUNT, Some(Arrow::Number(max_count))),
			],
		);
		Counter { clout }
	}
}

impl Writable for Counter {
	fn to_flights(&self) -> Vec<Flight> { self.clout.to_flights() }
}

impl<'a> CloutFilter<'a> for Counter {
	fn key_ring() -> &'a Ring { &COUNT }
	fn data_rings() -> &'a [&'a Ring] { &[&COUNT, &MAX_COUNT] }
	fn from_name_and_properties(target: &Target, properties: Vec<(&Ring, Option<Arrow>)>) -> Self {
		let clout = Clout::new(target, properties);
		Counter { clout }
	}
}

#[test]
fn filter() {
	let counter = Counter::new("card-counter", 7, 56);
	let mut chamber = {
		let echo = Echo::connect(&unique_name("x-holder"), &std::env::temp_dir());
		echo.write(|txn| txn.writable(&counter)).unwrap();
		echo.chamber().unwrap()
	};
	let counters = chamber.clouts::<Counter>().unwrap();
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
				write.attributes(vec![(&COUNT, Arrow::Number(1))])
			})
		})
	};
	let job2 = {
		let (tx, rx) = channel::<Echo>();
		let job = thread::spawn(move || {
			for echo in rx {
				echo.write(|write| {
					write.attributes(vec![(&MAX_COUNT, Arrow::Number(100))])
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
			write.arrow(Arrow::Number(3));
		})?;
		path
	};
	{
		let echo = Echo::connect(&path, &std::env::temp_dir());
		echo.write(|write| {
			write.arrow(Arrow::Number(10));
		})?;
	}
	let echo = Echo::connect(&path, &std::env::temp_dir());
	let mut chamber = echo.chamber()?;
	assert_eq!(chamber.arrow_or_none(), Some(Arrow::Number(10)));
	Ok(())
}

#[test]
fn reconnect() -> Result<(), Box<dyn Error>> {
	let path = {
		let path = unique_name("echo-test-");
		let echo = Echo::connect(&path, &std::env::temp_dir());
		echo.write(|write| {
			write.arrow(Arrow::Number(3));
			write.arrow(Arrow::Number(10));
		})?;
		path
	};
	let echo = Echo::connect(&path, &std::env::temp_dir());
	let mut chamber = echo.chamber()?;
	assert_eq!(chamber.arrow_or_none(), Some(Arrow::Number(10)));
	Ok(())
}

#[test]
fn targets_with_ring() -> Result<(), Box<dyn Error>> {
	let dracula = Target::new("Dracula");
	let bo_peep = Target::new("Bo Peep");
	let echo = Echo::connect(&unique_name("echo-test-"), &std::env::temp_dir());
	echo.write(|shout| {
		shout.write_target_properties(&dracula, vec![(&COUNT, Arrow::Number(3))]);
		shout.write_target_properties(&bo_peep, vec![(&COUNT, Arrow::Number(7))]);
	})?;
	let mut targets = echo.chamber()?.targets_with_ring(&COUNT)?;
	targets.sort();
	assert_eq!(targets, vec![bo_peep, dracula]);
	Ok(())
}

#[test]
fn target_attributes() -> Result<(), Box<dyn Error>> {
	let dracula = Target::String("Dracula".into());
	let echo = Echo::connect(&unique_name("echo-test-"), &std::env::temp_dir());
	echo.write(|shout| {
		shout.write_target_properties(&dracula, vec![(&COUNT, Arrow::Number(3))]);
	})?;
	let attributes = echo.chamber()?.arrows_at_target_rings(&dracula, vec![&COUNT]);
	assert_eq!(attributes.get(&COUNT), Some(&Arrow::Number(3)));
	Ok(())
}

#[test]
fn attributes() -> Result<(), Box<dyn Error>> {
	let echo = Echo::connect(&unique_name("echo-test-"), &std::env::temp_dir());
	echo.write(|shout| {
		shout.attributes(vec![
			(&MAX_COUNT, Arrow::Number(100)),
			(&COUNT, Arrow::Number(0)),
		]);
	})?;
	let attributes = echo.chamber()?.properties(vec![&MAX_COUNT, &COUNT]);
	assert_eq!(attributes, vec![
		(&MAX_COUNT, Some(Arrow::Number(100))),
		(&COUNT, Some(Arrow::Number(0))),
	]);
	Ok(())
}

#[test]
fn arrow() -> Result<(), Box<dyn Error>> {
	let echo = Echo::connect(&unique_name("echo-test-"), &std::env::temp_dir());
	let mut old_chamber = echo.chamber()?;
	echo.write(|write| {
		write.arrow(Arrow::Number(3))
	})?;
	let mut new_chamber = echo.chamber()?;
	assert_eq!(new_chamber.arrow_or_none(), Some(Arrow::Number(3)));
	assert_eq!(old_chamber.arrow_or_none(), None);
	Ok(())
}
