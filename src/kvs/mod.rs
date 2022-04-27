//! Open a key-value store, assign values to keys.
//! ```
//! use echodb::kvs;
//! let store = kvs::open("my-store", &std::env::temp_dir());
//! ```
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io;
use std::io::ErrorKind;
use std::path::Path;

use crate::{Chamber, Echo, ObjectId, Point, Target};

/// Read values at keys.
pub struct Catalog {
	chamber: Chamber
}

/// Write values to keys and acquire catalogs.
pub struct Store {
	echo: Echo
}

pub trait Key: Hash {}

pub trait Value: Sized {
	fn to_value_string(&self) -> String;
	fn from_value_string(s: &String) -> Result<Self, Box<dyn Error>>;
}

pub fn open(name: &str, folder: &Path) -> Result<Store, Box<dyn Error>> {
	//! Open a key-value store with the given name in the specified folder.
	let echo = Echo::connect(name, folder);
	Ok(Store { echo })
}

impl Catalog {
	pub fn read<K: Key, V: Value, F: Fn() -> V>(&self, key: &K, fallback: F) -> Result<V, Box<dyn Error>> {
		//! Read the value at key.
		let object_id = key_object_id(key);
		let target = self.chamber.target_at_object_point_or_none(&object_id, &VALUE_POINT);
		match target {
			None => Ok(fallback()),
			Some(ref target) => if let Target::String(ref s) = target {
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
		self.echo.write(|echo_writer| {
			let object_id = key_object_id(key);
			echo_writer.write_object_properties(&object_id, vec![
				(&VALUE_POINT, Target::String(value.to_value_string()))
			]);
		})?;
		Ok(())
	}
	pub fn catalog(&self) -> Result<Catalog, Box<dyn Error>> {
		//! Acquire a reader for the current state of the store.
		let chamber = self.echo.chamber()?;
		Ok(Catalog { chamber })
	}
}

const VALUE_POINT: Point = Point::Static { aspect: "echo::kv", name: "value" };

fn key_object_id<K: Key>(key: &K) -> ObjectId {
	let key_hash = (key_hash(key) as i64).abs();
	let string = format!("key-{}", key_hash);
	ObjectId::String(string)
}

fn key_hash<K: Key>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}



