use std::io;

use echo_lib::{Chamber, Echo, ObjectId, Point, Target};

pub const BLOG_TITLE: &Point = &Point::Static { aspect: "Blog", name: "title" };
pub const BLOG_OWNER: &Point = &Point::Static { aspect: "Blog", name: "owner" };

pub fn create_if_none(blogger_id: &ObjectId, echo: &Echo) -> io::Result<ObjectId> {
	let old_blog_id = read(blogger_id, &echo.chamber()?)?;
	let blog_id = match old_blog_id {
		Some(id) => id.clone(),
		None => echo.write(|write| {
			let blog_id = write.new_object_id("blog");
			write.write_object_properties(&blog_id, vec![
				(BLOG_OWNER, Target::Object(blogger_id.to_owned())),
				(BLOG_TITLE, Target::String("Musings".to_string()))
			]);
			blog_id
		})?,
	};
	Ok(blog_id)
}

pub fn read(blogger_id: &ObjectId, chamber: &Chamber) -> io::Result<Option<ObjectId>> {
	let blogs = chamber.objects_with_property(BLOG_OWNER, &Target::Object(blogger_id.clone()))?;
	Ok(blogs.first().cloned())
}
