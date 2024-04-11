use crate::util::datom_tree::{Datom, EphemeralNodeElement, node_map, Trie};

pub fn inject_datom(datom: Datom, prefix: &u8, node_map: &u32, node_elements: &Vec<EphemeralNodeElement>) -> Trie {
	let element = EphemeralNodeElement::Datom(datom.clone());
	inject_element(element, *prefix, *node_map, node_elements)
}

pub fn inject_trie(trie: Trie, prefix: u8, node_map: u32, node_elements: &Vec<EphemeralNodeElement>) -> Trie {
	let element = EphemeralNodeElement::Trie(trie);
	inject_element(element, prefix, node_map, node_elements)
}

fn inject_element(element: EphemeralNodeElement, prefix: u8, node_map: u32, node_elements: &Vec<EphemeralNodeElement>) -> Trie {
	let mut new_element = Some(element);
	let mut new_node_map = [false; 32];
	let mut new_node_elements = Vec::new();
	let elements_indexes = node_map::expand(node_map);
	for i in 0..32u8 {
		if i == prefix {
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

