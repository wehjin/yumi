use crate::util::datom_tree::Datom;

pub struct EavKey([u8; 24]);

impl From<&Datom> for EavKey {
	fn from(datom: &Datom) -> Self {
		let e = datom.e.to_key_parts();
		let a = datom.a.to_key_parts();
		let v = datom.v.to_key_parts();
		let parts = [
			e[6], e[5], e[4], e[3], e[2], e[1], e[0],
			a[3], a[2], a[1], a[0],
			v[12], v[11], v[10], v[9], v[8], v[7], v[6], v[5], v[4], v[3], v[2], v[1], v[0],
		];
		EavKey(parts)
	}
}

impl EavKey {
	const E6: usize = 23;
	pub fn prefix0(&self) -> u8 {
		self.0[Self::E6]
	}
}
