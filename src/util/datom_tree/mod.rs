use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::util::datom_tree::keys::EavKey;

#[cfg(test)]
mod tests;
pub(crate) mod keys;
mod node_map;

pub struct DatomTree {
	transactions_count: usize,
	trie: Rc<Trie>,
}

impl DatomTree {
	pub fn new() -> Self {
		let trie = Rc::new(Trie::empty());
		DatomTree { trie, transactions_count: 0 }
	}
	pub fn entity(&self, entity_id: EntityId) -> Entity {
		Entity { entity_id, trie: self.trie.clone(), attributes_ids: RefCell::new(None) }
	}
}

impl DatomTree {
	pub fn transact(mut self, steps: &[Step]) -> Self {
		let tc = self.transactions_count;
		let t = TransactionId(tc);
		let datoms = steps.iter().map(|step| step.to_datom(t)).collect::<Vec<_>>();
		let datom = datoms.first().unwrap().clone();
		self.trie = Rc::new(Trie::from_datom(datom));
		self.transactions_count = tc + 1;
		self
	}
}

pub struct Entity {
	entity_id: EntityId,
	trie: Rc<Trie>,
	attributes_ids: RefCell<Option<Rc<HashSet<AttributeId>>>>,
}

impl Entity {
	pub fn attribute_ids(&self) -> Rc<HashSet<AttributeId>> {
		if self.attributes_ids.borrow().is_none() {
			let datoms = self.trie.search(self.entity_id);
			let set = datoms.iter().map(Datom::attribute_id).cloned().collect::<HashSet<_>>();
			let _ = self.attributes_ids.borrow_mut().insert(Rc::new(set));
		}
		self.attributes_ids.borrow().as_ref().expect("datom-set").clone()
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct EntityId(u32);

impl EntityId {
	pub fn to_key_parts(&self) -> [u8; 7] {
		return [
			(self.0 >> 30) as u8 & 0b11111,
			(self.0 >> 25) as u8 & 0b11111,
			(self.0 >> 20) as u8 & 0b11111,
			(self.0 >> 15) as u8 & 0b11111,
			(self.0 >> 10) as u8 & 0b11111,
			(self.0 >> 5) as u8 & 0b11111,
			(self.0 >> 0) as u8 & 0b11111
		];
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AttributeId(u16);

impl AttributeId {
	pub fn to_key_parts(&self) -> [u8; 4] {
		return [
			(self.0 >> 15) as u8 & 0b11111,
			(self.0 >> 10) as u8 & 0b11111,
			(self.0 >> 5) as u8 & 0b11111,
			(self.0 >> 0) as u8 & 0b11111
		];
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Value(u64);

impl Value {
	pub fn to_key_parts(&self) -> [u8; 13] {
		return [
			(self.0 >> 60) as u8 & 0b11111,
			(self.0 >> 55) as u8 & 0b11111,
			(self.0 >> 50) as u8 & 0b11111,
			(self.0 >> 45) as u8 & 0b11111,
			(self.0 >> 40) as u8 & 0b11111,
			(self.0 >> 35) as u8 & 0b11111,
			(self.0 >> 30) as u8 & 0b11111,
			(self.0 >> 25) as u8 & 0b11111,
			(self.0 >> 20) as u8 & 0b11111,
			(self.0 >> 15) as u8 & 0b11111,
			(self.0 >> 10) as u8 & 0b11111,
			(self.0 >> 5) as u8 & 0b11111,
			(self.0 >> 0) as u8 & 0b11111
		];
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Effect { Insert, Eject }

#[derive(Debug, Copy, Clone)]
pub struct Step {
	pub e: EntityId,
	pub a: AttributeId,
	pub v: Value,
	pub f: Effect,
}

impl Step {
	pub fn to_datom(&self, t: TransactionId) -> Datom {
		Datom {
			e: self.e.clone(),
			a: self.a.clone(),
			v: self.v.clone(),
			_t: t,
			f: self.f.clone(),
		}
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TransactionId(usize);

impl TransactionId {
	pub fn to_u64(&self) -> u64 {
		self.0.clone() as u64
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Datom {
	e: EntityId,
	a: AttributeId,
	v: Value,
	_t: TransactionId,
	f: Effect,
}

impl Datom {
	pub fn attribute_id(&self) -> &AttributeId { &self.a }
}

#[derive(Debug, Clone)]
enum Trie {
	Ephemeral {
		node_map: u32,
		node_elements: Vec<EphemeralNodeElement>,
	}
}

impl Trie {
	pub fn search(&self, entity_id: EntityId) -> HashSet<Datom> {
		let mut datoms = HashSet::new();
		let mut current_trie = self;
		for prefix in entity_id.to_key_parts() {
			let element = current_trie.lookup(prefix);
			match element {
				EphemeralNodeElement::Datom(datom) => {
					if datom.e == entity_id {
						datoms.insert(datom.clone());
					}
					break;
				}
				EphemeralNodeElement::Trie(trie) => {
					current_trie = trie;
				}
			}
		}
		datoms
	}

	fn lookup(&self, key: u8) -> &EphemeralNodeElement {
		match self {
			Trie::Ephemeral { node_map, node_elements } => {
				let array_index = node_map::array_index(key, *node_map);
				&node_elements[array_index]
			}
		}
	}

	pub fn from_datom(datom: Datom) -> Self {
		let eav_key = EavKey::from(&datom);
		let key_prefix = eav_key.prefix0();
		let node_map = node_map::map_entry(key_prefix);
		let node_element = EphemeralNodeElement::Datom(datom);
		Self::Ephemeral { node_map, node_elements: vec![node_element] }
	}

	pub fn empty() -> Self {
		Self::Ephemeral { node_map: 0, node_elements: vec![] }
	}
}

#[derive(Debug, Clone)]
enum EphemeralNodeElement {
	Datom(Datom),
	Trie(Trie),
}
