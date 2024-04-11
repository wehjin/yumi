use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::util::datom_tree::keys::EavKey;

#[cfg(test)]
mod tests;
pub(crate) mod keys;
mod node_map;
mod trie;

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
		let transactions_count = self.transactions_count;
		let tx_id = TransactionId(transactions_count);
		let datoms = steps.iter().map(|step| step.to_datom(tx_id)).collect::<Vec<_>>();
		for datom in datoms {
			self.trie = Rc::new(self.trie.append(datom));
		}
		self.transactions_count = transactions_count + 1;
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
			match current_trie.lookup(prefix) {
				None => break,
				Some(element) => {
					match element {
						EphemeralNodeElement::Datom(datom) => {
							if datom.e == entity_id && datom.f == Effect::Insert {
								datoms.insert(datom.clone());
							}
							break;
						}
						EphemeralNodeElement::Trie(trie) => {
							current_trie = trie;
						}
					}
				}
			}
		}
		datoms
	}

	fn lookup(&self, key: u8) -> Option<&EphemeralNodeElement> {
		match self {
			Trie::Ephemeral { node_map, node_elements } => {
				match node_map::array_index(key, *node_map) {
					None => None,
					Some(index) => {
						let element = &node_elements[index];
						Some(&element)
					}
				}
			}
		}
	}
}

struct Eavtf;

impl Eavtf {
	pub fn keys_eq(&self, datom1: &Datom, datom2: &Datom) -> bool {
		datom1.e == datom2.e && datom1.a == datom2.a && datom1.v == datom2.v
	}
	pub fn get_prefixes(&self, datom: &Datom) -> Vec<u8> {
		EavKey::from(datom).prefixes().to_vec()
	}
}

impl Trie {
	pub fn append(&self, datom: Datom) -> Self {
		let trie_policy = Eavtf;
		let prefixes = trie_policy.get_prefixes(&datom);
		let mut current_prefixes = prefixes.as_slice();
		let mut current_trie = self;
		let mut back_trie: Trie;
		let mut back_tasks: Vec<(u32, &Vec<EphemeralNodeElement>, u8)> = Vec::new();
		loop {
			match current_prefixes.first() {
				None => unreachable!("out of prefixes"),
				Some(prefix) =>
					match current_trie {
						Trie::Ephemeral { node_map, node_elements } =>
							match current_trie.lookup(*prefix) {
								None => {
									back_trie = trie::inject_datom(datom, prefix, node_map, node_elements);
									break;
								}
								Some(element) => match element {
									EphemeralNodeElement::Datom(old_datom) => {
										match trie_policy.keys_eq(&datom, old_datom) {
											true => {
												back_trie = trie::inject_datom(datom, prefix, node_map, node_elements);
												break;
											}
											false => {
												unimplemented!("move both datoms to a new node")
											}
										}
									}
									EphemeralNodeElement::Trie(trie) => {
										back_tasks.push((*node_map, node_elements, *prefix));
										current_trie = trie;
										current_prefixes = &current_prefixes[1..];
									}
								},
							},
					},
			}
		}
		while let Some((node_map, node_elements, prefix)) = back_tasks.pop() {
			back_trie = trie::inject_trie(back_trie, prefix, node_map, node_elements);
		}
		back_trie
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
