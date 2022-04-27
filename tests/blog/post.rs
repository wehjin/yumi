use std::io;

use recurvedb::{Arrow, Chamber, Recurve, Ring, Target};

pub const TITLE: &Ring = &Ring::Static { aspect: "BlogPost", name: "title" };
pub const BODY: &Ring = &Ring::Static { aspect: "BlogPost", name: "body" };
pub const BLOG_ID: &Ring = &Ring::Static { aspect: "BlogPost", name: "blog" };

pub fn read_ordered(blog: &Target, chamber: &Chamber) -> io::Result<Vec<Target>> {
	let mut posts = chamber.targets_with_property(BLOG_ID, &Arrow::Target(blog.to_owned()))?;
	posts.sort_by_key(|it| chamber.string(it, TITLE));
	Ok(posts)
}

pub fn create(title: &str, body: &str, blog: &Target, recurve: &Recurve) -> io::Result<Target> {
	recurve.write(|write| {
		let post = write.new_target("blog-post");
		write.write_target_properties(&post, vec![
			(TITLE, Arrow::String(title.to_string())),
			(BODY, Arrow::String(body.to_string())),
			(BLOG_ID, Arrow::Target(blog.to_owned())),
		]);
		post
	})
}
