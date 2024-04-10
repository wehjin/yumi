use crate::util::datom_tree::{AttributeId, Change, DatomTree, EntityId, Step, TransactionId, Value};

#[test]
fn add_one_datom() {
	let datom_tree = DatomTree::new().transact(&[
		Step { e: EntityId(1), a: AttributeId(20), v: Value(5), c: Change::Insert }
	]);
	let entity = datom_tree.entity(EntityId(1));
	assert_eq!(1, entity.attribute_ids().len())
}

#[test]
fn transaction_id_to_number() {
	let tx_id = TransactionId(5);
	assert_eq!(5, tx_id.to_u64())
}