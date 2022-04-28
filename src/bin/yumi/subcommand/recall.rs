use std::error::Error;

use clap::{Arg, ArgMatches, Command};

use recurvedb::string_ring_at_divider;

pub const COMMAND_NAME: &'static str = "recall";

pub fn cli() -> Command<'static> {
	Command::new(COMMAND_NAME)
		.about("Pull data from the database")
		.arg_required_else_help(true)
		.arg(Arg::new("TARGET").required(true).index(1))
		.arg(Arg::new("RING").required(true).index(2))
}

pub fn main(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
	let target = args.value_of("TARGET").expect("target");
	let ring = args.value_of("RING").expect("ring");

	let recurve = super::connect();
	let bundle = recurve.to_bundle()?;
	let target = target.into();
	let ring = string_ring_at_divider(ring, '/');
	let arrow = bundle.arrow_at_target_ring_or_none(&target, &ring);
	if let Some(arrow) = arrow {
		println!("{}", arrow.to_string())
	}
	Ok(())
}
