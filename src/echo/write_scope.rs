use crate::{Target, Ring, Flight, Archer, Arrow, Writable};
use crate::util::unique_name;

/// WriteScope allows a function to write facts into the database.
pub struct WriteScope {
	pub flights: Vec<Flight>
}

impl WriteScope {
	pub fn new_target(&self, prefix: &str) -> Target { Target::String(unique_name(prefix)) }

	pub fn writable(&mut self, writable: &impl Writable) {
		self.flights(writable.to_flights())
	}

	pub fn write_target_properties(&mut self, target: &Target, properties: Vec<(&Ring, Arrow)>) {
		for (ring, arrow) in properties {
			let flight = Flight { archer: Archer::Unit, target: target.to_owned(), ring: ring.to_owned(), arrow: Some(arrow) };
			self.flights.push(flight)
		}
	}

	pub fn attributes(&mut self, attributes: Vec<(&Ring, Arrow)>) {
		self.write_target_properties(&Target::Unit, attributes)
	}

	pub fn arrow(&mut self, arrow: Arrow) {
		self.attributes(vec![(&Ring::Unit, arrow)])
	}

	fn flights(&mut self, flight: Vec<Flight>) {
		self.flights.extend(flight);
	}
}
