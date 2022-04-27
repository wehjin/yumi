use std::{io, thread};
use std::error::Error;
use std::sync::mpsc::channel;

use recurvedb::{Arrow, Clout, CloutFilter, Flight, Recurve, Ring, Target, CanVolley};
use recurvedb::util::unique_name;

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

impl CanVolley for Counter {
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
		let recurve = Recurve::connect(&unique_name("x-holder"), &std::env::temp_dir());
		recurve.draw(|txn| txn.release(&counter)).unwrap();
		recurve.chamber().unwrap()
	};
	let counters = chamber.clouts::<Counter>().unwrap();
	assert_eq!(1, counters.len());
	let final_counter = &counters[0];
	assert_eq!(final_counter, &counter);
	assert_eq!(final_counter.count(), 7)
}

#[test]
fn multi_thread() -> Result<(), Box<dyn Error>> {
	let recurve = Recurve::connect(&unique_name("test-multi-thread"), &std::env::temp_dir());
	let job1 = {
		let recurve = recurve.clone();
		thread::spawn(move || {
			recurve.draw(|write| {
				write.release_unit_attributes(vec![(&COUNT, Arrow::Number(1))])
			})
		})
	};
	let job2 = {
		let (tx, rx) = channel::<Recurve>();
		let job = thread::spawn(move || {
			for recurve in rx {
				recurve.draw(|write| {
					write.release_unit_attributes(vec![(&MAX_COUNT, Arrow::Number(100))])
				}).unwrap();
			}
			Ok(()) as io::Result<()>
		});
		tx.send(recurve.clone()).unwrap();
		job
	};
	job1.join().unwrap()?;
	job2.join().unwrap()?;
	let chamber = recurve.chamber()?;
	let attributes = chamber.properties(vec![&COUNT, &MAX_COUNT]);
	assert_eq!(attributes.len(), 2);
	Ok(())
}

#[test]
fn double_reconnect() -> Result<(), Box<dyn Error>> {
	let path = {
		let path = unique_name("recurve-test-");
		let recurve = Recurve::connect(&path, &std::env::temp_dir());
		recurve.draw(|write| {
			write.release_unit_center_ring_arrow(Arrow::Number(3));
		})?;
		path
	};
	{
		let recurve = Recurve::connect(&path, &std::env::temp_dir());
		recurve.draw(|write| {
			write.release_unit_center_ring_arrow(Arrow::Number(10));
		})?;
	}
	let recurve = Recurve::connect(&path, &std::env::temp_dir());
	let mut chamber = recurve.chamber()?;
	assert_eq!(chamber.arrow_or_none(), Some(Arrow::Number(10)));
	Ok(())
}

#[test]
fn reconnect() -> Result<(), Box<dyn Error>> {
	let path = {
		let path = unique_name("recurve-test-");
		let recurve = Recurve::connect(&path, &std::env::temp_dir());
		recurve.draw(|write| {
			write.release_unit_center_ring_arrow(Arrow::Number(3));
			write.release_unit_center_ring_arrow(Arrow::Number(10));
		})?;
		path
	};
	let recurve = Recurve::connect(&path, &std::env::temp_dir());
	let mut chamber = recurve.chamber()?;
	assert_eq!(chamber.arrow_or_none(), Some(Arrow::Number(10)));
	Ok(())
}

#[test]
fn targets_with_ring() -> Result<(), Box<dyn Error>> {
	let dracula = Target::new("Dracula");
	let bo_peep = Target::new("Bo Peep");
	let recurve = Recurve::connect(&unique_name("recurve-test-"), &std::env::temp_dir());
	recurve.draw(|shout| {
		shout.release_target_properties(&dracula, vec![(&COUNT, Arrow::Number(3))]);
		shout.release_target_properties(&bo_peep, vec![(&COUNT, Arrow::Number(7))]);
	})?;
	let mut targets = recurve.chamber()?.targets_with_ring(&COUNT)?;
	targets.sort();
	assert_eq!(targets, vec![bo_peep, dracula]);
	Ok(())
}

#[test]
fn target_attributes() -> Result<(), Box<dyn Error>> {
	let dracula = Target::String("Dracula".into());
	let recurve = Recurve::connect(&unique_name("recurve-test-"), &std::env::temp_dir());
	recurve.draw(|shout| {
		shout.release_target_properties(&dracula, vec![(&COUNT, Arrow::Number(3))]);
	})?;
	let attributes = recurve.chamber()?.arrows_at_target_rings(&dracula, vec![&COUNT]);
	assert_eq!(attributes.get(&COUNT), Some(&Arrow::Number(3)));
	Ok(())
}

#[test]
fn attributes() -> Result<(), Box<dyn Error>> {
	let recurve = Recurve::connect(&unique_name("recurve-test-"), &std::env::temp_dir());
	recurve.draw(|shout| {
		shout.release_unit_attributes(vec![
			(&MAX_COUNT, Arrow::Number(100)),
			(&COUNT, Arrow::Number(0)),
		]);
	})?;
	let attributes = recurve.chamber()?.properties(vec![&MAX_COUNT, &COUNT]);
	assert_eq!(attributes, vec![
		(&MAX_COUNT, Some(Arrow::Number(100))),
		(&COUNT, Some(Arrow::Number(0))),
	]);
	Ok(())
}

#[test]
fn arrow() -> Result<(), Box<dyn Error>> {
	let recurve = Recurve::connect(&unique_name("recurve-test-"), &std::env::temp_dir());
	let mut old_chamber = recurve.chamber()?;
	recurve.draw(|write| {
		write.release_unit_center_ring_arrow(Arrow::Number(3))
	})?;
	let mut new_chamber = recurve.chamber()?;
	assert_eq!(new_chamber.arrow_or_none(), Some(Arrow::Number(3)));
	assert_eq!(old_chamber.arrow_or_none(), None);
	Ok(())
}
