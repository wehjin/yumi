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

	use crate::{Echo, Point, Sayer, Object, Target};

	#[test]
	fn main() -> Result<(), Box<dyn Error>> {
		let sayer = Sayer::Named("Bob".into());
		let object = Object::Sayer(sayer.clone());
		let point = Point::NameAspect("counter".into(), "Count".into());
		let mut echo = Echo::connect();
		let mut chamber = echo.chamber()?;
		let mut new_chamber = echo.batch_write(|ctx| {
			ctx.say(&sayer, &object, &point, &Target::Number(3))
		})?;
		assert_eq!(new_chamber.full_read(&sayer, &object, &point), Some(Target::Number(3)));
		assert_eq!(chamber.full_read(&sayer, &object, &point), None);
		Ok(())
	}

	#[test]
	fn unit_attributes() -> Result<(), Box<dyn Error>> {
		let max_count: Point = ("max_count", "Counter").into();
		let count: Point = ("count", "Counter").into();
		let mut echo = Echo::connect();
		let mut chamber = echo.unit_attributes(vec![
			(&max_count, Target::Number(100)),
			(&count, Target::Number(0))
		])?;
		let attributes = chamber.unit_attributes(vec![&max_count, &count]);
		assert_eq!(attributes, vec![
			(&max_count, Some(Target::Number(100))),
			(&count, Some(Target::Number(0)))
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
