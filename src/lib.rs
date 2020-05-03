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
	use crate::{Melody, Nova, Say, Ship, Singer, Spin, Subject};

	#[test]
	fn main() {
		let ray = Nova::connect().latest();
		let new_ray = ray.origin().beam(|ctx| {
			let singer = Singer::Named("Bob".to_string());
			let subject = Subject::Singer(singer.clone());
			let ship = Ship::Static("counter", "Count");
			let object = Say::Number(3);
			let spin = Spin::Up;
			ctx.add(Melody::Up(singer, subject, ship, object));
		});
	}
}
