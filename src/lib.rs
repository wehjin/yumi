extern crate rand;

pub use self::beam::*;
pub use self::chamber::*;
pub use self::core::*;
pub use self::echo::Echo;

mod beam;
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

	use crate::{Echo, Object, Point, Sayer, Target};

	const COUNT: Point = Point::Static { name: "count", aspect: "Counter" };
	const MAX_COUNT: Point = Point::Static { name: "max_count", aspect: "Counter" };

	#[test]
	fn main() -> Result<(), Box<dyn Error>> {
		let sayer = Sayer::Named("Bob".into());
		let object = Object::Sayer(sayer.clone());
		let mut echo = Echo::connect();
		let mut chamber = echo.chamber()?;
		let mut new_chamber = echo.batch_write(|ctx| {
			ctx.say(&sayer, &object, &COUNT, &Target::Number(3))
		})?;
		assert_eq!(new_chamber.full_read(&sayer, &object, &COUNT), Some(Target::Number(3)));
		assert_eq!(chamber.full_read(&sayer, &object, &COUNT), None);
		Ok(())
	}

	#[test]
	fn unit_attributes() -> Result<(), Box<dyn Error>> {
		let mut echo = Echo::connect();
		let mut chamber = echo.unit_attributes(vec![
			(&MAX_COUNT, Target::Number(100)),
			(&COUNT, Target::Number(0))
		])?;
		let attributes = chamber.unit_attributes(vec![&MAX_COUNT, &COUNT]);
		assert_eq!(attributes, vec![
			(&MAX_COUNT, Some(Target::Number(100))),
			(&COUNT, Some(Target::Number(0)))
		]);
		Ok(())
	}

	#[test]
	fn unit_target() -> Result<(), Box<dyn Error>> {
		let mut echo = Echo::connect();
		let mut chamber = echo.chamber()?;
		let mut new_chamber = echo.unit_target(Target::Number(3))?;
		assert_eq!(new_chamber.unit_target(), Some(Target::Number(3)));
		assert_eq!(chamber.unit_target(), None);
		Ok(())
	}
}
