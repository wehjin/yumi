use std::error::Error;
use std::path::Path;

use clap::{ArgMatches, Command};

use echodb::kvs;

mod write;
mod read;

pub fn cli() -> Command<'static> {
	Command::new("kv")
		.about("Manage the database's key-value store")
		.subcommand_required(true)
		.arg_required_else_help(true)
		.subcommand(write::cli())
		.subcommand(read::cli())
}

pub fn main(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
	match args.subcommand() {
		Some(("write", sub_matches)) => write::main(sub_matches),
		Some(("read", sub_matches)) => read::main(sub_matches),
		_ => unreachable!(),
	}
}

fn open_kvs() -> Result<kvs::Store, Box<dyn Error>> {
	let kvs = echodb::kvs::open("canyon", Path::new("."))?;
	Ok(kvs)
}


#[derive(Debug, Clone, PartialEq, Hash)]
pub struct CanyonKey(String);

impl kvs::Key for CanyonKey {}

#[derive(Debug, Clone, PartialEq)]
pub struct CanyonString(String);

impl kvs::Value for CanyonString {
	fn to_value_string(&self) -> String {
		self.0.to_string()
	}

	fn from_value_string(s: &String) -> Result<Self, Box<dyn Error>> {
		Ok(CanyonString(s.to_string()))
	}
}