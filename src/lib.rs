extern crate rand;

pub use self::chamber::*;
pub use self::core::*;
pub use self::echo::Echo;

mod chamber;
mod core;
mod echo;
mod util;
pub mod hamt;
pub mod diary;
pub mod bytes;

#[cfg(test)]
mod tests {
	use std::{io, thread};
	use std::collections::HashMap;
	use std::error::Error;
	use std::sync::mpsc::channel;

	use crate::{Echo, ObjName, Point, Say, Sayer, Target, util, Writable};

	const COUNT: Point = Point::Static { name: "count", aspect: "Counter" };
	const MAX_COUNT: Point = Point::Static { name: "max_count", aspect: "Counter" };

	#[derive(Debug, Eq, PartialEq)]
	struct Counter {
		obj_name: ObjName,
		attributes: HashMap<Point, Target>,
	}

	impl Counter {
		pub fn new(name: &str, count: u64, max_count: u64) -> Self {
			let obj_name = ObjName::String(name.into());
			let mut attributes = HashMap::new();
			attributes.insert(COUNT, Target::Number(count));
			attributes.insert(MAX_COUNT, Target::Number(max_count));
			Counter { obj_name, attributes }
		}
	}

	impl Writable for Counter {
		fn to_says(&self) -> Vec<Say> {
			self.attributes.keys()
				.map(|point| Say {
					sayer: Sayer::Unit,
					object: self.obj_name.to_owned(),
					point: point.to_owned(),
					target: self.attributes.get(point).map(Target::to_owned),
				})
				.collect()
		}
	}

	trait PointHolder<'a> {
		fn key_point() -> &'a Point;
		fn data_points() -> &'a [&'a Point];
		fn from_name_and_attributes(obj_name: &ObjName, attributes: Vec<(&Point, Option<Target>)>) -> Self;
	}

	const COUNTER_POINTS: &[&Point] = &[&COUNT, &MAX_COUNT];

	impl<'a> PointHolder<'a> for Counter {
		fn key_point() -> &'a Point { COUNTER_POINTS[0] }
		fn data_points() -> &'a [&'a Point] { COUNTER_POINTS }
		fn from_name_and_attributes(obj_name: &ObjName, attributes: Vec<(&Point, Option<Target>)>) -> Self {
			let mut map = HashMap::new();
			for (point, target) in attributes {
				if let Some(target) = target {
					map.insert(point.to_owned(), target);
				}
			}
			Counter { obj_name: obj_name.to_owned(), attributes: map }
		}
	}

	#[test]
	fn point_holder() {
		let counter = Counter::new("card-counter", 1, 56);
		let mut chamber = {
			let echo = Echo::connect(&util::temp_dir("point-holder").unwrap());
			echo.write(|txn| txn.writable(&counter)).unwrap();
			echo.chamber().unwrap()
		};
		let obj_names = chamber.objects_with_point(Counter::key_point()).unwrap();
		let found_counters = obj_names.into_iter().map(|obj_name| {
			let attributes = chamber.object_attributes(&obj_name, Counter::data_points().to_vec());
			Counter::from_name_and_attributes(&obj_name, attributes)
		}).collect::<Vec<_>>();
		assert_eq!(1, found_counters.len());
		assert_eq!(counter, found_counters[0]);
	}

	#[test]
	fn multi_thread() -> Result<(), Box<dyn Error>> {
		let echo = Echo::connect(&util::temp_dir("test-multi-thread")?);
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
		let mut chamber = echo.chamber()?;
		let attributes = chamber.attributes(vec![&COUNT, &MAX_COUNT]);
		assert_eq!(attributes.len(), 2);
		Ok(())
	}

	#[test]
	fn double_reconnect() -> Result<(), Box<dyn Error>> {
		let path = {
			let path = util::temp_dir("echo-test-")?;
			let echo = Echo::connect(&path);
			echo.write(|write| {
				write.target(Target::Number(3));
			})?;
			path
		};
		{
			let echo = Echo::connect(&path);
			echo.write(|write| {
				write.target(Target::Number(10));
			})?;
		}
		let echo = Echo::connect(&path);
		let mut chamber = echo.chamber()?;
		assert_eq!(chamber.target(), Some(Target::Number(10)));
		Ok(())
	}

	#[test]
	fn reconnect() -> Result<(), Box<dyn Error>> {
		let path = {
			let path = util::temp_dir("echo-test-")?;
			let echo = Echo::connect(&path);
			echo.write(|write| {
				write.target(Target::Number(3));
				write.target(Target::Number(10));
			})?;
			path
		};
		let echo = Echo::connect(&path);
		let mut chamber = echo.chamber()?;
		assert_eq!(chamber.target(), Some(Target::Number(10)));
		Ok(())
	}

	#[test]
	fn objects_with_point() -> Result<(), Box<dyn Error>> {
		let dracula = ObjName::new("Dracula");
		let bo_peep = ObjName::new("Bo Peep");
		let echo = Echo::connect(&util::temp_dir("echo-test-")?);
		echo.write(|shout| {
			shout.object_attributes(&dracula, vec![(&COUNT, Target::Number(3)), ]);
			shout.object_attributes(&bo_peep, vec![(&COUNT, Target::Number(7)), ]);
		})?;
		let mut objects = echo.chamber()?.objects_with_point(&COUNT)?;
		objects.sort();
		assert_eq!(objects, vec![bo_peep, dracula]);
		Ok(())
	}

	#[test]
	fn object_attributes() -> Result<(), Box<dyn Error>> {
		let dracula = ObjName::String("Dracula".into());
		let echo = Echo::connect(&util::temp_dir("echo-test-")?);
		echo.write(|shout| {
			shout.object_attributes(&dracula, vec![(&COUNT, Target::Number(3))]);
		})?;
		let attributes = echo.chamber()?.object_attributes(&dracula, vec![&COUNT]);
		assert_eq!(attributes[0], (&COUNT, Some(Target::Number(3))));
		Ok(())
	}

	#[test]
	fn attributes() -> Result<(), Box<dyn Error>> {
		let echo = Echo::connect(&util::temp_dir("echo-test-")?);
		echo.write(|shout| {
			shout.attributes(vec![
				(&MAX_COUNT, Target::Number(100)),
				(&COUNT, Target::Number(0))
			]);
		})?;
		let attributes = echo.chamber()?.attributes(vec![&MAX_COUNT, &COUNT]);
		assert_eq!(attributes, vec![
			(&MAX_COUNT, Some(Target::Number(100))),
			(&COUNT, Some(Target::Number(0)))
		]);
		Ok(())
	}

	#[test]
	fn target() -> Result<(), Box<dyn Error>> {
		let echo = Echo::connect(&util::temp_dir("echo-test-")?);
		let mut old_chamber = echo.chamber()?;
		echo.write(|write| {
			write.target(Target::Number(3))
		})?;
		let mut new_chamber = echo.chamber()?;
		assert_eq!(new_chamber.target(), Some(Target::Number(3)));
		assert_eq!(old_chamber.target(), None);
		Ok(())
	}
}
