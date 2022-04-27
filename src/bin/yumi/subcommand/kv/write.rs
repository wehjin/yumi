use std::error::Error;

use clap::{Arg, ArgMatches, Command};

use crate::kv::{YumiKey, YumiString};

pub fn cli() -> Command<'static> {
	Command::new("write")
		.about("Write a key value pair into the database")
		.arg_required_else_help(true)
		.arg(Arg::new("KEY").required(true).index(1))
		.arg(Arg::new("VALUE").required(true).index(2))
}

pub fn main(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
	let key = YumiKey(args.value_of("KEY").expect("key").to_string());
	let value = YumiString(args.value_of("VALUE").expect("value").to_string());
	let kvs = super::open_kvs()?;
	kvs.write(&key, &value)?;
	Ok(())
}
