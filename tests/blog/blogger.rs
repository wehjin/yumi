use std::io;

use echodb::{Chamber, Echo, ObjectId, Point, Target};

pub const NAME: &Point = &Point::Static { aspect: "Blogger", name: "name" };


pub fn create_if_none(echo: &Echo) -> io::Result<ObjectId> {
	let old_blogger_id = read(&echo.chamber()?)?;
	let blogger_id = match old_blogger_id {
		Some(id) => id.clone(),
		None => echo.write(|write| {
			let blogger_id = write.new_object_id("blogger");
			write.write_object_properties(&blogger_id, vec![
				(NAME, Target::String("Alice".to_string()))
			]);
			blogger_id
		})?,
	};
	Ok(blogger_id)
}

pub fn read(chamber: &Chamber) -> io::Result<Option<ObjectId>> {
	let bloggers = chamber.objects_with_point(NAME)?;
	Ok(bloggers.first().cloned())
}
