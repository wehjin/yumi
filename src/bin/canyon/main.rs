extern crate clap;
extern crate serde;
extern crate serde_derive;
extern crate toml;

use std::error::Error;

use clap::Command;

use crate::subcommand::{init, kv};

mod subcommand;
mod settings;

fn cli() -> Command<'static> {
	Command::new("canyon")
		.about("Manage an echo database")
		.subcommand_required(true)
		.arg_required_else_help(true)
		.subcommand(init::cli())
		.subcommand(kv::cli())
}

fn main() -> Result<(), Box<dyn Error>> {
	let matches = cli().get_matches();
	match matches.subcommand() {
		Some(("init", _sub_matches)) => init::main()?,
		Some(("kv", sub_matches)) => kv::main(sub_matches)?,
		_ => unreachable!(),
	}
	Ok(())
}
