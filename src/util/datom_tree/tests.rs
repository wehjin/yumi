use crate::util::datom_tree::{AttributeId, DatomTree, Effect, EntityId, Step, TransactionId, Value};

#[test]
fn add_one_datom_find_no_attributes_for_entity_with_similar_prefix() {
	let datom_tree = DatomTree::new().transact(&[
		Step { e: EntityId(1), a: AttributeId(20), v: Value(5), f: Effect::Insert }
	]);
	let entity = datom_tree.entity(EntityId(33));
	assert_eq!(0, entity.attribute_ids().len())
}

#[test]
fn add_one_datom_find_attribute_for_entity() {
	let datom_tree = DatomTree::new().transact(&[
		Step { e: EntityId(1), a: AttributeId(20), v: Value(5), f: Effect::Insert }
	]);
	let entity = datom_tree.entity(EntityId(1));
	assert_eq!(1, entity.attribute_ids().len())
}

#[test]
fn transaction_id_to_number() {
	let tx_id = TransactionId(5);
	assert_eq!(5, tx_id.to_u64())
}