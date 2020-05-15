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
	use std::error::Error;

	use crate::{Echo, ObjName, Point, Target};

	const COUNT: Point = Point::Static { name: "count", aspect: "Counter" };
	const MAX_COUNT: Point = Point::Static { name: "max_count", aspect: "Counter" };

	#[test]
	fn objects_with_point() -> Result<(), Box<dyn Error>> {
		let dracula = ObjName::new("Dracula");
		let bo_peep = ObjName::new("Bo Peep");
		let mut echo = Echo::connect();
		echo.shout(|shout| {
			shout.object_attributes(&dracula, vec![(&COUNT, Target::Number(3)), ]);
			shout.object_attributes(&bo_peep, vec![(&COUNT, Target::Number(7)), ]);
		})?;
		let objects = echo.chamber()?.objects_with_point(&COUNT)?;
		assert_eq!(objects, vec![dracula, bo_peep].into_iter().collect());
		Ok(())
	}

	#[test]
	fn object_attributes() -> Result<(), Box<dyn Error>> {
		let dracula = ObjName::String("Dracula".into());
		let mut echo = Echo::connect();
		echo.shout(|shout| {
			shout.object_attributes(&dracula, vec![(&COUNT, Target::Number(3))]);
		})?;
		let attributes = echo.chamber()?.object_attributes(&dracula, vec![&COUNT])[0];
		assert_eq!(attributes, (&COUNT, Some(Target::Number(3))));
		Ok(())
	}

	#[test]
	fn attributes() -> Result<(), Box<dyn Error>> {
		let mut echo = Echo::connect();
		echo.shout(|shout| {
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
		let mut echo = Echo::connect();
		let mut old_chamber = echo.chamber()?;
		echo.shout(|write| {
			write.target(Target::Number(3))
		})?;
		let mut new_chamber = echo.chamber()?;
		assert_eq!(new_chamber.target(), Some(Target::Number(3)));
		assert_eq!(old_chamber.target(), None);
		Ok(())
	}
}
