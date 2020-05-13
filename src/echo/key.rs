use std::hash::{Hash, Hasher};

use crate::{hamt, Sayer, Point, Object};

#[cfg(test)]
mod tests {
	use crate::{Sayer, Point, Object};
	use crate::echo::EchoKey;
	use crate::hamt::Key;

	#[test]
	fn hash() {
		let key = EchoKey::SayerObjjectPoint(Sayer::Unit, Object::Unit, Point::Unit);
		let hash = key.universal(1);
		assert!(hash < 0x80000000)
	}
}

pub enum EchoKey {
	SayerObjjectPoint(Sayer, Object, Point)
}

impl hamt::Key for EchoKey {}

impl Hash for EchoKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			EchoKey::SayerObjjectPoint(sayer, object, point) => {
				sayer.hash(state);
				state.write(DIVIDER);
				object.hash(state);
				state.write(DIVIDER);
				point.hash(state);
			}
		}
	}
}

const DIVIDER: &[u8] = &['/' as u8];
