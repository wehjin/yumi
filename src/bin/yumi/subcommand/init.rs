use std::error::Error;
use std::fs::OpenOptions;
use std::io;
use std::io::{ErrorKind, Write};

use clap::Command;
use rand::random;

use crate::settings::{IngressSettings, Settings};


pub fn cli() -> Command<'static> {
	Command::new("init")
		.about("Initialize a database in the current directory")
}

pub fn main() -> Result<(), Box<dyn Error>> {
	let settings = Settings {
		ingress: IngressSettings { user_codes: vec![random()] }
	};
	let settings = toml::to_string_pretty(&settings).expect("toml for Settings");
	let path = "Yumi.toml";
	let mut file = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(path)
		.map_err(|e|
			if let ErrorKind::AlreadyExists = e.kind() {
				io::Error::new(ErrorKind::AlreadyExists, format!("{} exists", path))
			} else { e }
		)?;
	file.write_all(settings.as_bytes())?;
	Ok(())
}

