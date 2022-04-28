use std::error::Error;

use clap::{Arg, ArgMatches, Command};

use recurvedb::{Archer, Flight, string_arrow_if_not_empty, string_ring_at_divider, Target};

pub const COMMAND_NAME: &'static str = "release";

pub fn cli() -> Command<'static> {
	Command::new(COMMAND_NAME)
		.about("Release data into the database")
		.arg_required_else_help(true)
		.arg(Arg::new("TARGET").required(true).index(1))
		.arg(Arg::new("RING").required(true).index(2))
		.arg(Arg::new("ARROW").required(true).index(3))
}

pub fn main(args: &ArgMatches) -> Result<(), Box<dyn Error>> {
	let target = args.value_of("TARGET").expect("target");
	let ring = args.value_of("RING").expect("ring");
	let arrow = args.value_of("ARROW").expect("arrow");
	let recurve = super::connect();
	recurve.draw(|scope| {
		let flight = Flight {
			archer: Archer::Unit,
			target: Target::new(target),
			ring: string_ring_at_divider(ring, '/'),
			arrow: string_arrow_if_not_empty(arrow),
		};
		scope.release(&flight);
	})?;
	Ok(())
}
