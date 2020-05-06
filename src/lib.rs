extern crate rand;

pub use self::beam::*;
pub use self::chamber::*;
pub use self::core::*;
pub use self::echo::Echo;

mod beam;
mod chamber;
mod core;
mod echo;
mod hamt;
mod mem_file;
mod util;
pub mod diary;
pub mod write_bytes;

#[cfg(test)]
mod tests {
	use std::error::Error;

	use crate::{Echo, Said, Sayer, Ship, Subject};

	#[test]
	fn main() -> Result<(), Box<dyn Error>> {
		let sayer = Sayer::Named("Bob".into());
		let subject = Subject::Sayer(sayer.clone());
		let ship = Ship::Static("counter", "Count");
		let chamber = Echo::connect().latest()?;
		let new_chamber = chamber.origin()
			.batch_write(|ctx| {
				ctx.say(&sayer, &subject, &ship, &Said::Number(3))
			})?;
		assert_eq!(new_chamber.full_read(&sayer, &subject, &ship), Some(&Said::Number(3)));
		assert_eq!(chamber.full_read(&sayer, &subject, &ship), None);
		Ok(())
	}

	#[test]
	fn said() -> Result<(), Box<dyn Error>> {
		let chamber = Echo::connect().latest()?;
		let new_chamber = chamber.origin().write(Said::Number(3))?;
		assert_eq!(new_chamber.read(), Some(&Said::Number(3)));
		assert_eq!(chamber.read(), None);
		Ok(())
	}
}
