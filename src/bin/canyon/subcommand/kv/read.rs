use std::error::Error;

use clap::{Arg, ArgMatches, Command};
use echodb::kvs::Value;

use crate::kv::{CanyonKey, CanyonString};

pub fn cli() -> Command<'static> {
	Command::new("read")
		.about("Read a key value pair's value from the database")
		.arg_required_else_help(true)
		.arg(Arg::new("KEY").required(true).index(1))
}

pub fn main(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
	let key = CanyonKey(args.value_of("KEY").expect("key").to_string());
	let kvs = super::open_kvs()?;
	let catalog = kvs.catalog()?;
	let value = catalog.read(&key, || CanyonString("fail".into()))?;
	println!("{}", value.to_value_string());
	Ok(())
}
