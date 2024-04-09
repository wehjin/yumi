use crate::util::datom_tree::{Change, DatomTree, Step};

#[test]
fn it_works() {
	let datom_tree = DatomTree::new()
		.transact(&[
			Step { e: 1, a: 20, v: 5, c: Change::Insert }
		]);
	let entity = datom_tree.entity(1);
	assert_eq!(1, entity.attribute_ids().len())
}
