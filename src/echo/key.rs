use std::hash::{Hash, Hasher};

use crate::{hamt, Sayer, Ship, Subject};

#[cfg(test)]
mod tests {
	use crate::{Sayer, Ship, Subject};
	use crate::echo::EchoKey;
	use crate::hamt::Key;

	#[test]
	fn hash() {
		let key = EchoKey::SayerSubjectShip(Sayer::Unit, Subject::Unit, Ship::Unit);
		let hash = key.universal(1);
		assert!(hash < 0x80000000)
	}
}

pub enum EchoKey {
	SayerSubjectShip(Sayer, Subject, Ship)
}

impl hamt::Key for EchoKey {}

impl Hash for EchoKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			EchoKey::SayerSubjectShip(sayer, subject, ship) => {
				sayer.hash(state);
				state.write(DIVIDER);
				subject.hash(state);
				state.write(DIVIDER);
				ship.hash(state);
			}
		}
	}
}

const DIVIDER: &[u8] = &['/' as u8];
