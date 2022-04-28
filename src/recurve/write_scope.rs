use crate::{Archer, Arrow, CanVolley, Flight, Ring, Target};
use crate::util::unique_name;

/// `DrawScope` allows a function to release `Arrows` into `Targets`.
pub struct DrawScope {
	pub flights: Vec<Flight>,
}

impl DrawScope {
	pub fn new_target(&self, prefix: &str) -> Target { Target::String(unique_name(prefix)) }

	pub fn release(&mut self, can_volley: &impl CanVolley) {
		self.release_flights(can_volley.to_flights())
	}

	// TODO Find a better name for properties
	pub fn release_target_properties(&mut self, target: &Target, properties: Vec<(&Ring, Arrow)>) {
		for (ring, arrow) in properties {
			let flight = Flight { archer: Archer::Unit, target: target.to_owned(), ring: ring.to_owned(), arrow: Some(arrow) };
			self.flights.push(flight)
		}
	}

	// TODO Delete this method or move elsewhere
	pub fn release_unit_attributes(&mut self, attributes: Vec<(&Ring, Arrow)>) {
		self.release_target_properties(&Target::Unit, attributes)
	}

	// TODO Delete this method or move elsewhere
	pub fn release_unit_center_ring_arrow(&mut self, arrow: Arrow) {
		self.release_unit_attributes(vec![(&Ring::Center, arrow)])
	}

	fn release_flights(&mut self, flights: Vec<Flight>) {
		self.flights.extend(flights);
	}
}
