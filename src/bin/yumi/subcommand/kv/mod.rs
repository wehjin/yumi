use std::error::Error;
use std::path::Path;

use clap::{ArgMatches, Command};

use recurvedb::kvs;

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
	let kvs = recurvedb::kvs::open("yumi", Path::new("."))?;
	Ok(kvs)
}


#[derive(Debug, Clone, PartialEq, Hash)]
pub struct YumiKey(String);

impl kvs::Key for YumiKey {}

#[derive(Debug, Clone, PartialEq)]
pub struct YumiString(String);

impl kvs::Value for YumiString {
	fn to_value_string(&self) -> String {
		self.0.to_string()
	}

	fn from_value_string(s: &String) -> Result<Self, Box<dyn Error>> {
		Ok(YumiString(s.to_string()))
	}
}