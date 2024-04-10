use crate::util::datom_tree::Datom;

#[cfg(test)]
mod tests {
	use crate::util::datom_tree::{AttributeId, Datom, Effect, EntityId, TransactionId, Value};
	use crate::util::datom_tree::keys::EavKey;

	#[test]
	fn eav_key_basic() {
		let eav_key = EavKey::from(&Datom {
			e: EntityId(1),
			a: AttributeId(2),
			v: Value(4),
			_t: TransactionId(8),
			f: Effect::Insert,
		});
		let prefixes = eav_key.prefixes();
		let selected = [
			prefixes[0],
			prefixes[6],
			prefixes[7],
			prefixes[10],
		];
		assert_eq!([0, 1, 0, 2], selected);
	}
}

pub struct EavKey([u8; 24]);

impl From<&Datom> for EavKey {
	fn from(datom: &Datom) -> Self {
		let e = datom.e.to_key_parts();
		let a = datom.a.to_key_parts();
		let v = datom.v.to_key_parts();
		let parts = [
			e[0], e[1], e[2], e[3], e[4], e[5], e[6],
			a[0], a[1], a[2], a[3],
			v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11], v[12],
		];
		EavKey(parts)
	}
}

impl EavKey {
	pub fn prefixes(&self) -> &[u8] { &self.0 }
}
