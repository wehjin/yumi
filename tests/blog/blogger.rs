use std::io;

use recurvedb::{Chamber, Recurve, Target, Ring, Arrow};

pub const NAME: &Ring = &Ring::Static { aspect: "Blogger", name: "name" };


pub fn create_if_none(recurve: &Recurve) -> io::Result<Target> {
	let old_blogger = read(&recurve.chamber()?)?;
	let blogger = match old_blogger {
		Some(target) => target.clone(),
		None => recurve.write(|write| {
			let blogger = write.new_target("blogger");
			write.write_target_properties(&blogger, vec![
				(NAME, Arrow::String("Alice".to_string()))
			]);
			blogger
		})?,
	};
	Ok(blogger)
}

pub fn read(chamber: &Chamber) -> io::Result<Option<Target>> {
	let bloggers = chamber.targets_with_ring(NAME)?;
	Ok(bloggers.first().cloned())
}
