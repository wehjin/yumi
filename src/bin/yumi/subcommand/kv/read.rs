use std::error::Error;

use clap::{Arg, ArgMatches, Command};
use recurvedb::kvs::Value;

use crate::kv::{YumiKey, YumiString};

pub fn cli() -> Command<'static> {
	Command::new("read")
		.about("Read a key value pair's value from the database")
		.arg_required_else_help(true)
		.arg(Arg::new("KEY").required(true).index(1))
}

pub fn main(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
	let key = YumiKey(args.value_of("KEY").expect("key").to_string());
	let kvs = super::open_kvs()?;
	let catalog = kvs.catalog()?;
	let value = catalog.read(&key, || YumiString("fail".into()))?;
	println!("{}", value.to_value_string());
	Ok(())
}
