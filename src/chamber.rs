use std::collections::HashMap;
use std::io;

use crate::{Arrow, diary, Ring, Target};
use crate::hamt::{Hamt, ProdAB, Reader, Root};

pub struct Chamber {
	pub(crate) target_rings_reader: Reader,
	pub(crate) ring_targets_reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}


impl Chamber {
	pub fn clouts<'a, F: CloutFilter<'a>>(&mut self) -> io::Result<Vec<F>> {
		let targets = self.targets_with_ring(F::key_ring())?;
		let clouts = targets.into_iter()
			.map(|target| {
				let properties = self.target_properties(&target, F::data_rings().to_vec());
				F::from_name_and_properties(&target, properties)
			}).collect::<Vec<_>>();
		Ok(clouts)
	}

	pub fn targets_with_property(&self, ring: &Ring, arrow: &Arrow) -> io::Result<Vec<Target>> {
		let mut matching_targets = Vec::new();
		for target in self.targets_with_ring(ring)? {
			if arrow.eq(&self.read_arrow(&target, ring)?.unwrap()) {
				matching_targets.push(target)
			}
		}
		Ok(matching_targets)
	}

	pub fn targets_with_ring(&self, ring: &Ring) -> io::Result<Vec<Target>> {
		let mut diary_reader = self.diary_reader.clone();
		self.inner_targets_with_ring(ring, &mut diary_reader)
	}

	fn inner_targets_with_ring(&self, ring: &Ring, reader: &mut diary::Reader) -> io::Result<Vec<Target>> {
		let targets_root: Option<Root> = self.ring_targets_reader.read_value(ring, reader)?;
		let targets = match targets_root {
			None => Vec::new(),
			Some(root) => {
				let target_arrow_reader = Hamt::new(root).reader()?;
				let target_arrow = target_arrow_reader.read_all::<ProdAB<Target, Arrow>>(reader)?;
				target_arrow.into_iter().map(|it| it.a).collect()
			}
		};
		Ok(targets)
	}

	fn target_properties<'a>(&self, target: &'a Target, rings: Vec<&'a Ring>) -> Vec<(&'a Ring, Option<Arrow>)> {
		rings.into_iter().map(|ring| {
			let arrow = self.read_arrow(target, ring).unwrap_or(None);
			(ring, arrow)
		}).collect()
	}

	pub fn properties<'a>(&self, rings: Vec<&'a Ring>) -> Vec<(&'a Ring, Option<Arrow>)> {
		self.target_properties(&Target::Unit, rings)
	}

	pub fn string(&self, target: &Target, ring: &Ring) -> String {
		self.arrow_at_target_ring(target, ring).as_str().to_string()
	}

	pub fn number(&self, target: &Target, ring: &Ring) -> u64 {
		self.arrow_at_target_ring(target, ring).as_number()
	}

	pub fn target(&self, target: &Target, ring: &Ring) -> Target {
		self.arrow_at_target_ring(target, ring).as_target().to_owned()
	}

	pub fn arrows_at_target_rings(&self, target: &Target, rings: Vec<&Ring>) -> HashMap<Ring, Arrow> {
		let mut map = HashMap::new();
		for (ring, arrow) in self.target_properties(target, rings) {
			if let Some(arrow) = arrow {
				map.insert(ring.to_owned(), arrow);
			}
		}
		map
	}

	pub fn arrow_at_target_ring(&self, target: &Target, ring: &Ring) -> Arrow {
		let option = self.arrow_at_target_ring_or_none(target, ring);
		option.unwrap()
	}

	pub fn arrow_at_target_ring_or_none(&self, target: &Target, ring: &Ring) -> Option<Arrow> {
		//! Acquire some arrow at a ring on an target or nothing.
		let option = self.read_arrow(target, ring).unwrap();
		option
	}

	pub fn arrow_or_none(&mut self) -> Option<Arrow> {
		self.arrow_at_target_ring_or_none(&Target::Unit, &Ring::Center)
	}

	fn read_arrow(&self, target: &Target, ring: &Ring) -> io::Result<Option<Arrow>> {
		let mut reader = self.diary_reader.clone();
		let root: Option<Root> = self.target_rings_reader.read_value(target, &mut reader)?;
		match root {
			None => Ok(None),
			Some(root) => {
				let ring_arrows = Hamt::new(root);
				ring_arrows.reader()?.read_value(ring, &mut reader)
			}
		}
	}
}

pub trait CloutFilter<'a> {
	fn key_ring() -> &'a Ring;
	fn data_rings() -> &'a [&'a Ring];
	fn from_name_and_properties(target: &Target, properties: Vec<(&Ring, Option<Arrow>)>) -> Self;
}
