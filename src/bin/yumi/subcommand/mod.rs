use std::path::Path;

use recurvedb::Recurve;

pub mod init;
pub mod kv;
pub mod release;
pub mod recall;

fn connect() -> Recurve {
	let recurve = Recurve::connect("yumi", Path::new("."));
	recurve
}
