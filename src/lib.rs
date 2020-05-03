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

	use crate::{Nova, Said, Say, Sayer, Ship, Subject};

	#[test]
	fn main() -> Result<(), Box<dyn Error>> {
		let ray = Nova::connect().latest();
		let sayer = Sayer::Named("Bob".to_string());
		let subject = Subject::Singer(sayer.clone());
		let ship = Ship::Static("counter", "Count");
		let said = Said::Number(3);
		let new_ray = ray.origin().beam(|ctx| {
			ctx.say(Say::Assert(sayer, subject.to_owned(), ship.to_owned(), said.to_owned()));
		})?;
		let new_said = new_ray.said(&subject, &ship);
		assert_eq!(new_said, said);
		Ok(())
	}
}
