use serde_derive::Serialize;

#[derive(Serialize)]
pub struct Settings {
	pub version: Version,
}

#[derive(Serialize)]
pub struct Version {
	pub major: u32,
	pub minor: u32,
}
