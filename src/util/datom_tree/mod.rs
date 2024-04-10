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
		Entity { id: entity_id, trie: self.trie.clone() }
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
	id: EntityId,
	trie: Rc<Trie>,
}

impl Entity {
	pub fn attribute_ids(&self) -> Vec<AttributeId> {
		let e_key = EKey::from(self.id);
		let path = e_key.as_path();
		let element = self.trie.search(path);
		match element {
			EphemeralNodeElement::Datom(datom) => {
				match datom.c {
					Change::Insert => vec![datom.a.clone()],
					Change::Eject => vec![]
				}
			}
			EphemeralNodeElement::Trie(_) => vec![]
		}
	}
}

struct EKey([u8; 7]);

impl EKey {
	pub fn as_path(&self) -> &[u8] {
		&self.0
	}
}

impl From<EntityId> for EKey {
	fn from(entity_id: EntityId) -> Self {
		EKey(entity_id.to_key_parts())
	}
}


#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub enum Change { Insert, Eject }

#[derive(Debug, Copy, Clone)]
pub struct Step {
	pub e: EntityId,
	pub a: AttributeId,
	pub v: Value,
	pub c: Change,
}

impl Step {
	pub fn to_datom(&self, t: TransactionId) -> Datom {
		Datom {
			e: self.e.clone(),
			a: self.a.clone(),
			v: self.v.clone(),
			_t: t,
			c: self.c.clone(),
		}
	}
}

#[derive(Debug, Copy, Clone)]
pub struct TransactionId(usize);

impl TransactionId {
	pub fn to_u64(&self) -> u64 {
		self.0.clone() as u64
	}
}

#[derive(Debug, Copy, Clone)]
pub struct Datom {
	e: EntityId,
	a: AttributeId,
	v: Value,
	_t: TransactionId,
	c: Change,
}

#[derive(Debug, Clone)]
enum Trie {
	Ephemeral {
		node_map: u32,
		node_elements: Vec<EphemeralNodeElement>,
	}
}

impl Trie {
	pub fn search(&self, path: &[u8]) -> EphemeralNodeElement {
		let key = path[0];
		let element = self.lookup(key);
		element.clone()
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
