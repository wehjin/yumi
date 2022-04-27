use std::collections::HashMap;
use std::io;

use crate::{diary, ObjectId, Ring, Arrow};
use crate::hamt::{Hamt, ProdAB, Reader, Root};

pub struct Chamber {
	pub(crate) object_rings_reader: Reader,
	pub(crate) ring_objects_reader: Reader,
	pub(crate) diary_reader: diary::Reader,
}


impl Chamber {
	pub fn objects<'a, F: ObjectFilter<'a>>(&mut self) -> io::Result<Vec<F>> {
		let obj_names = self.objects_with_ring(F::key_ring())?;
		let objects = obj_names.into_iter()
			.map(|obj_name| {
				let properties = self.object_properties(&obj_name, F::data_rings().to_vec());
				F::from_name_and_properties(&obj_name, properties)
			}).collect::<Vec<_>>();
		Ok(objects)
	}

	pub fn objects_with_property(&self, ring: &Ring, arrow: &Arrow) -> io::Result<Vec<ObjectId>> {
		let mut matching_objects = Vec::new();
		for object in self.objects_with_ring(ring)? {
			if arrow.eq(&self.read_arrow(&object, ring)?.unwrap()) {
				matching_objects.push(object)
			}
		}
		Ok(matching_objects)
	}

	pub fn objects_with_ring(&self, ring: &Ring) -> io::Result<Vec<ObjectId>> {
		let mut diary_reader = self.diary_reader.clone();
		self.inner_objects_with_ring(ring, &mut diary_reader)
	}

	fn inner_objects_with_ring(&self, ring: &Ring, reader: &mut diary::Reader) -> io::Result<Vec<ObjectId>> {
		let objects_root: Option<Root> = self.ring_objects_reader.read_value(ring, reader)?;
		let objects = match objects_root {
			None => Vec::new(),
			Some(root) => {
				let object_arrow_reader = Hamt::new(root).reader()?;
				let object_arrow = object_arrow_reader.read_all::<ProdAB<ObjectId, Arrow>>(reader)?;
				object_arrow.into_iter().map(|it| it.a).collect()
			}
		};
		Ok(objects)
	}

	fn object_properties<'a>(&self, object: &'a ObjectId, rings: Vec<&'a Ring>) -> Vec<(&'a Ring, Option<Arrow>)> {
		rings.into_iter().map(|ring| {
			let arrow = self.read_arrow(object, ring).unwrap_or(None);
			(ring, arrow)
		}).collect()
	}

	pub fn properties<'a>(&self, rings: Vec<&'a Ring>) -> Vec<(&'a Ring, Option<Arrow>)> {
		self.object_properties(&ObjectId::Unit, rings)
	}

	pub fn string(&self, object: &ObjectId, ring: &Ring) -> String {
		self.arrow_at_object_ring(object, ring).as_str().to_string()
	}

	pub fn number(&self, object: &ObjectId, ring: &Ring) -> u64 {
		self.arrow_at_object_ring(object, ring).as_number()
	}

	pub fn object_id(&self, object: &ObjectId, ring: &Ring) -> ObjectId {
		self.arrow_at_object_ring(object, ring).as_object_id().to_owned()
	}

	pub fn arrows_at_object_rings(&self, object: &ObjectId, rings: Vec<&Ring>) -> HashMap<Ring, Arrow> {
		let mut map = HashMap::new();
		for (ring, arrow) in self.object_properties(object, rings) {
			if let Some(arrow) = arrow {
				map.insert(ring.to_owned(), arrow);
			}
		}
		map
	}

	pub fn arrow_at_object_ring(&self, object: &ObjectId, ring: &Ring) -> Arrow {
		let option = self.arrow_at_object_ring_or_none(object, ring);
		option.unwrap()
	}

	pub fn arrow_at_object_ring_or_none(&self, object: &ObjectId, ring: &Ring) -> Option<Arrow> {
		//! Acquire some arrow at a ring on an object or nothing.
		let option = self.read_arrow(object, ring).unwrap();
		option
	}

	pub fn arrow_or_none(&mut self) -> Option<Arrow> {
		self.arrow_at_object_ring_or_none(&ObjectId::Unit, &Ring::Unit)
	}

	fn read_arrow(&self, object: &ObjectId, ring: &Ring) -> io::Result<Option<Arrow>> {
		let mut reader = self.diary_reader.clone();
		let root: Option<Root> = self.object_rings_reader.read_value(object, &mut reader)?;
		match root {
			None => Ok(None),
			Some(root) => {
				let ring_arrows = Hamt::new(root);
				ring_arrows.reader()?.read_value(ring, &mut reader)
			}
		}
	}
}

pub trait ObjectFilter<'a> {
	fn key_ring() -> &'a Ring;
	fn data_rings() -> &'a [&'a Ring];
	fn from_name_and_properties(obj_name: &ObjectId, properties: Vec<(&Ring, Option<Arrow>)>) -> Self;
}
