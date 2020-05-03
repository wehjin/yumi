pub use self::beam::*;
pub use self::core::*;
pub use self::nova::*;
pub use self::ray::*;

mod beam;
mod core;
mod hamt;
mod nova;
mod ray;

#[cfg(test)]
mod tests {
	use std::error::Error;

	use crate::{Nova, Said, Sayer, Ship, Subject};

	#[test]
	fn main() -> Result<(), Box<dyn Error>> {
		let sayer = Sayer::Named("Bob".into());
		let subject = Subject::Sayer(sayer.clone());
		let ship = Ship::Static("counter", "Count");
		let said = Said::Number(3);

		let ray = Nova::connect().latest()?;
		let new_ray = ray.origin().speak(|ctx| {
			ctx.say(&sayer, &subject, &ship, &said);
		})?;
		let new_said = new_ray.said(&subject, &ship).unwrap();
		assert_eq!(new_said, &said);
		Ok(())
	}
}
