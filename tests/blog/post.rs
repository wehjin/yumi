use std::io;

use echo_lib::{Chamber, Echo, ObjectId, Point, Target};

pub const TITLE: &Point = &Point::Static { aspect: "BlogPost", name: "title" };
pub const BODY: &Point = &Point::Static { aspect: "BlogPost", name: "body" };
pub const BLOG: &Point = &Point::Static { aspect: "BlogPost", name: "blog" };

pub fn read_ordered(blog_id: &ObjectId, chamber: &Chamber) -> io::Result<Vec<ObjectId>> {
	let mut posts = chamber.objects_with_property(BLOG, &Target::Object(blog_id.to_owned()))?;
	posts.sort_by_key(|it| chamber.string(it, TITLE));
	Ok(posts)
}

pub fn create(title: &str, body: &str, blog_id: &ObjectId, echo: &Echo) -> io::Result<ObjectId> {
	echo.write(|write| {
		let post_id = write.new_object_id("blog-post");
		write.write_object_properties(&post_id, vec![
			(TITLE, Target::String(title.to_string())),
			(BODY, Target::String(body.to_string())),
			(BLOG, Target::Object(blog_id.to_owned())),
		]);
		post_id
	})
}
