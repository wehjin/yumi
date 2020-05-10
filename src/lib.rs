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

	use crate::{Echo, Sayer, Ship, Subject, T};

	#[test]
	fn main() -> Result<(), Box<dyn Error>> {
		let sayer = Sayer::Named("Bob".into());
		let subject = Subject::Sayer(sayer.clone());
		let ship = Ship::FieldGroup("counter".into(), "Count".into());
		let mut echo = Echo::connect();
		let mut chamber = echo.latest()?;
		let mut new_chamber = echo.batch_write(|ctx| {
			ctx.say(&sayer, &subject, &ship, &T::Number(3))
		})?;
		assert_eq!(new_chamber.full_read(&sayer, &subject, &ship), Some(T::Number(3)));
		assert_eq!(chamber.full_read(&sayer, &subject, &ship), None);
		Ok(())
	}

	#[test]
	fn target() -> Result<(), Box<dyn Error>> {
		let mut echo = Echo::connect();
		let mut chamber = echo.latest()?;
		let mut new_chamber = echo.write(T::Number(3))?;
		assert_eq!(new_chamber.read(), Some(T::Number(3)));
		assert_eq!(chamber.read(), None);
		Ok(())
	}
}
