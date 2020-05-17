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
	use std::error::Error;
	use std::sync::mpsc::channel;

	use crate::{Echo, ObjName, Point, Target, util};

	const COUNT: Point = Point::Static { name: "count", aspect: "Counter" };
	const MAX_COUNT: Point = Point::Static { name: "max_count", aspect: "Counter" };

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
		let attributes = echo.chamber()?.object_attributes(&dracula, vec![&COUNT])[0];
		assert_eq!(attributes, (&COUNT, Some(Target::Number(3))));
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
