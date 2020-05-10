use std::hash::{Hash, Hasher};

use crate::{hamt, Sayer, Point, Subject};

#[cfg(test)]
mod tests {
	use crate::{Sayer, Point, Subject};
	use crate::echo::EchoKey;
	use crate::hamt::Key;

	#[test]
	fn hash() {
		let key = EchoKey::SayerSubjectPoint(Sayer::Unit, Subject::Unit, Point::Main);
		let hash = key.universal(1);
		assert!(hash < 0x80000000)
	}
}

pub enum EchoKey {
	SayerSubjectPoint(Sayer, Subject, Point)
}

impl hamt::Key for EchoKey {}

impl Hash for EchoKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			EchoKey::SayerSubjectPoint(sayer, subject, point) => {
				sayer.hash(state);
				state.write(DIVIDER);
				subject.hash(state);
				state.write(DIVIDER);
				point.hash(state);
			}
		}
	}
}

const DIVIDER: &[u8] = &['/' as u8];
