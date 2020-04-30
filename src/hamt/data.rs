use std::env;
use std::path::PathBuf;

pub fn temp_path(name: &str) -> PathBuf {
	let mut dir = env::temp_dir();
	dir.push(name);
	dir
}
