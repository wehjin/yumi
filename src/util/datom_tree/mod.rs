#[cfg(test)]
mod tests;

pub struct DatomTree {
	segment_store: SegmentStore,
	trie_ref: TrieRef,
	transactions_count: usize,
}

impl DatomTree {
	pub fn new() -> Self {
		let mut segment_store = SegmentStore::new();
		let node_map = segment_store.append_element(SegmentElement::NodeMap(0));
		let node_ref = segment_store.append_element(SegmentElement::Node(Vec::new()));
		let trie_ref = TrieRef { node_map_ref: node_map, node_ref };
		DatomTree { segment_store, trie_ref, transactions_count: 0 }
	}
	pub fn attributes(&self) -> usize {
		return 0;
	}
	pub fn count_transactions(&self) -> usize {
		return self.transactions_count;
	}
}

impl DatomTree {
	pub fn transact(mut self, steps: &[Step]) -> Self {
		let id = self.transactions_count;
		for step in steps {
			Self::transact_step(&mut self.segment_store, step, id)
		}
		self.transactions_count = id + 1;
		self
	}

	fn transact_step(segments: &mut SegmentStore, step: &Step, t: usize) {
		let eavt_key = step.to_eavt_key(t);
	}

	pub fn entity(&self, entity_id: EntityId) -> Entity {
		Entity { id: entity_id }
	}
}

pub struct Entity {
	id: EntityId,
}

impl Entity {
	pub fn attribute_ids(&self) -> Vec<AttributeId> { Vec::new() }
}

struct SegmentStore {
	ephemeral_elements: Vec<SegmentElement>,
}

impl SegmentStore {
	pub fn new() -> Self {
		SegmentStore {
			ephemeral_elements: Vec::new(),
		}
	}

	pub fn append_element(&mut self, element: SegmentElement) -> BlockRef {
		let offset = self.ephemeral_elements.len();
		self.ephemeral_elements.push(element);
		BlockRef {
			segment_ref: SegmentRef::Ephemeral,
			segment_offset: offset,
		}
	}
}

enum SegmentElement {
	NodeMap(u32),
	Node(Vec<NodeElement>),
}

pub type EntityId = u64;

pub type AttributeId = u64;
pub type Value = u64;

#[derive(Debug, Copy, Clone)]
pub enum Change { Insert, Eject }

#[derive(Debug, Copy, Clone)]
struct Step {
	e: EntityId,
	a: AttributeId,
	v: Value,
	c: Change,
}

impl Step {
	pub fn to_eavt_key(&self, transaction: usize) -> EavtKey {
		EavtKey(self.clone(), transaction)
	}
}

struct EavtKey(Step, usize);


struct TrieRef {
	pub node_map_ref: BlockRef,
	pub node_ref: BlockRef,
}

enum NodeElement {
	Content(BlockRef),
	Trie(TrieRef),
}

struct BlockRef {
	pub segment_ref: SegmentRef,
	pub segment_offset: usize,
}

enum SegmentRef {
	Ephemeral,
	Durable { store_index: usize },
}