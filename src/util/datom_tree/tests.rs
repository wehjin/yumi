use crate::util::datom_tree::{AttributeId, DatomTree, Effect, EntityId, Step, TransactionId, Value};

#[test]
fn inject_datoms_from_same_cohort_finds_attribute_for_both_entities() {
	let entity1 = EntityId(0b00001);
	let entity2 = EntityId(0b00010);
	let datom_tree = DatomTree::new().transact(&[
		Step { e: entity1, a: AttributeId(20), v: Value(1), f: Effect::Insert },
		Step { e: entity2, a: AttributeId(20), v: Value(1), f: Effect::Insert },
	]);
	let attributes1 = datom_tree.entity(entity1).attribute_ids();
	let attributes2 = datom_tree.entity(entity2).attribute_ids();
	let attribute_counts = [attributes1.len(), attributes2.len(), ];
	assert_eq!([1, 1], attribute_counts);
}

#[test]
fn inject_datoms_from_same_cohort_in_reverse_order_finds_attribute_for_both_entities() {
	let entity1 = EntityId(0b00001);
	let entity2 = EntityId(0b00010);
	let datom_tree = DatomTree::new().transact(&[
		Step { e: entity2, a: AttributeId(20), v: Value(2), f: Effect::Insert },
		Step { e: entity1, a: AttributeId(20), v: Value(2), f: Effect::Insert },
	]);
	let attributes1 = datom_tree.entity(entity1).attribute_ids();
	let attributes2 = datom_tree.entity(entity2).attribute_ids();
	let attribute_counts = [attributes1.len(), attributes2.len(), ];
	assert_eq!([1, 1], attribute_counts);
}

#[test]
fn inject_datoms_from_different_cohorts_finds_attribute_for_both_entities() {
	let entity1 = EntityId(0b000001);
	let entity2 = EntityId(0b100000);
	let datom_tree = DatomTree::new().transact(&[
		Step { e: entity1, a: AttributeId(20), v: Value(3), f: Effect::Insert },
		Step { e: entity2, a: AttributeId(20), v: Value(3), f: Effect::Insert },
	]);
	let attributes1 = datom_tree.entity(entity1).attribute_ids();
	let attributes2 = datom_tree.entity(entity2).attribute_ids();
	let attribute_counts = [attributes1.len(), attributes2.len(), ];
	assert_eq!([1, 1], attribute_counts);
}

#[test]
fn inject_datoms_from_different_cohorts_in_reverse_order_finds_attribute_for_both_entities() {
	let entity1 = EntityId(0b000001);
	let entity2 = EntityId(0b100000);
	let datom_tree = DatomTree::new().transact(&[
		Step { e: entity2, a: AttributeId(20), v: Value(4), f: Effect::Insert },
		Step { e: entity1, a: AttributeId(20), v: Value(4), f: Effect::Insert },
	]);
	let attributes1 = datom_tree.entity(entity1).attribute_ids();
	let attributes2 = datom_tree.entity(entity2).attribute_ids();
	let attribute_counts = [attributes1.len(), attributes2.len(), ];
	assert_eq!([1, 1], attribute_counts);
}

#[test]
fn eject_and_inject_same_datom_finds_attribute_for_entity() {
	let datom_tree = DatomTree::new().transact(&[
		Step { e: EntityId(3), a: AttributeId(20), v: Value(5), f: Effect::Eject },
		Step { e: EntityId(3), a: AttributeId(20), v: Value(5), f: Effect::Insert },
	]);
	let entity = datom_tree.entity(EntityId(3));
	assert_eq!(1, entity.attribute_ids().len())
}

#[test]
fn inject_and_eject_same_datom_finds_no_attribute_for_entity() {
	let datom_tree = DatomTree::new().transact(&[
		Step { e: EntityId(1), a: AttributeId(20), v: Value(5), f: Effect::Insert },
		Step { e: EntityId(1), a: AttributeId(20), v: Value(5), f: Effect::Eject },
	]);
	let entity = datom_tree.entity(EntityId(1));
	assert_eq!(0, entity.attribute_ids().len())
}

#[test]
fn add_one_datom_finds_no_attributes_for_entity_with_similar_prefix() {
	let datom_tree = DatomTree::new().transact(&[
		Step { e: EntityId(1), a: AttributeId(20), v: Value(5), f: Effect::Insert }
	]);
	let entity = datom_tree.entity(EntityId(33));
	assert_eq!(0, entity.attribute_ids().len())
}

#[test]
fn add_one_datom_finds_attribute_for_entity() {
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

#[test]
fn entity_prefixes() {
	let entity_id = EntityId(1);
	let prefixes = entity_id.to_key();
	assert_eq!([0, 0, 0, 0, 0, 0, 1], prefixes);
}