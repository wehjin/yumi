use crate::util::datom_tree::{Datom, Eavtf, EphemeralNodeElement, node_map, Trie};

pub fn zip_datoms(datom1: &Datom, datom1_key: &[u8], datom2: &Datom, datom2_start_depth: usize, policy: Eavtf) -> Trie {
	let datom2_key = &policy.get_key(datom2)[datom2_start_depth..];
	let mut back_trie: Trie;
	let mut back_tasks = Vec::new();
	let mut key_index = 0usize;
	loop {
		let datom1_array_index = datom1_key[key_index];
		let datom2_array_index = datom2_key[key_index];
		match datom2_array_index == datom1_array_index {
			true => {
				back_tasks.push(datom1_array_index);
				key_index += 1;
			}
			false => {
				let flag1 = node_map::key_flag(datom1_array_index);
				let flag2 = node_map::key_flag(datom2_array_index);
				let node_map = flag1 | flag2;
				let element1 = EphemeralNodeElement::Datom(datom1.clone());
				let element2 = EphemeralNodeElement::Datom(datom2.clone());
				let node_elements = match flag1 < flag2 {
					true => vec![element1, element2],
					false => vec![element2, element1],
				};
				back_trie = Trie::Ephemeral { node_map, node_elements };
				break;
			}
		}
	}
	while let Some(array_index) = back_tasks.pop() {
		let node_map = node_map::key_flag(array_index);
		let node_elements = vec![EphemeralNodeElement::Trie(back_trie)];
		back_trie = Trie::Ephemeral { node_map, node_elements }
	}
	back_trie
}

pub fn inject_datom(datom: Datom, array_index: &u8, node_map: &u32, node_elements: &Vec<EphemeralNodeElement>) -> Trie {
	let element = EphemeralNodeElement::Datom(datom.clone());
	inject_element(element, *array_index, *node_map, node_elements)
}

pub fn inject_trie(trie: Trie, array_index: u8, node_map: u32, node_elements: &Vec<EphemeralNodeElement>) -> Trie {
	let element = EphemeralNodeElement::Trie(trie);
	inject_element(element, array_index, node_map, node_elements)
}

fn inject_element(element: EphemeralNodeElement, array_index: u8, node_map: u32, node_elements: &Vec<EphemeralNodeElement>) -> Trie {
	let mut new_element = Some(element);
	let mut new_node_map = [false; 32];
	let mut new_node_elements = Vec::new();
	let elements_indexes = node_map::expand(node_map);
	for i in 0..32u8 {
		if i == array_index {
			new_node_elements.push(new_element.take().expect("element"));
			new_node_map[i as usize] = true;
		} else {
			if let Some(elements_index) = elements_indexes[i as usize] {
				let old_element = node_elements[elements_index].clone();
				new_node_elements.push(old_element);
				new_node_map[i as usize] = true;
			}
		}
	}
	Trie::Ephemeral {
		node_map: node_map::compress(new_node_map),
		node_elements: new_node_elements,
	}
}

