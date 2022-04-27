//! Open a key-value store, assign values to keys.
//! ```
//! use recurvedb::kvs;
//! let store = kvs::open("my-store", &std::env::temp_dir());
//! ```
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::ErrorKind;
use std::path::Path;

use crate::{Arrow, Chamber, Recurve, Ring, Target};

/// Read values at keys.
pub struct Catalog {
	chamber: Chamber,
}

/// Write values to keys and acquire catalogs.
pub struct Store {
	recurve: Recurve,
}

pub trait Key: Hash {}

pub trait Value: Sized {
	fn to_value_string(&self) -> String;
	fn from_value_string(s: &String) -> Result<Self, Box<dyn Error>>;
}

pub fn open(name: &str, folder: &Path) -> Result<Store, Box<dyn Error>> {
	//! Open a key-value store with the given name in the specified folder.
	let recurve = Recurve::connect(name, folder);
	Ok(Store { recurve })
}

impl Catalog {
	pub fn read<K: Key, V: Value, F: Fn() -> V>(&self, key: &K, fallback: F) -> Result<V, Box<dyn Error>> {
		//! Read the value at key.
		let target = key_target(key);
		let arrow = self.chamber.arrow_at_target_ring_or_none(&target, &VALUE_RING);
		match arrow {
			None => Ok(fallback()),
			Some(ref arrow) => if let Arrow::String(ref s) = arrow {
				let value = V::from_value_string(s)?;
				Ok(value)
			} else {
				Err(io::Error::from(ErrorKind::InvalidData).into())
			},
		}
	}
}

impl Store {
	pub fn write(&self, key: &impl Key, value: &impl Value) -> Result<(), Box<dyn Error>> {
		//! Assign a value to a key.
		self.recurve.draw(|scope| {
			let target = key_target(key);
			scope.release_target_properties(&target, vec![
				(&VALUE_RING, Arrow::String(value.to_value_string()))
			]);
		})?;
		Ok(())
	}
	pub fn catalog(&self) -> Result<Catalog, Box<dyn Error>> {
		//! Acquire a reader for the current state of the store.
		let chamber = self.recurve.chamber()?;
		Ok(Catalog { chamber })
	}
}

const VALUE_RING: Ring = Ring::Static { aspect: "recurve::kv", name: "value" };

fn key_target<K: Key>(key: &K) -> Target {
	let key_hash = (key_hash(key) as i64).abs();
	let string = format!("key-{}", key_hash);
	Target::String(string)
}

fn key_hash<K: Key>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}



