use std::io;

use echodb::{Arrow, Chamber, Echo, Ring, Target};

pub const BLOG_TITLE: &Ring = &Ring::Static { aspect: "Blog", name: "title" };
pub const BLOG_OWNER: &Ring = &Ring::Static { aspect: "Blog", name: "owner" };

pub fn create_if_none(blogger: &Target, echo: &Echo) -> io::Result<Target> {
	let old_blog = read(blogger, &echo.chamber()?)?;
	let blog = match old_blog {
		Some(target) => target.clone(),
		None => echo.write(|write| {
			let blog = write.new_target("blog");
			write.write_target_properties(&blog, vec![
				(BLOG_OWNER, Arrow::Target(blogger.to_owned())),
				(BLOG_TITLE, Arrow::String("Musings".to_string())),
			]);
			blog
		})?,
	};
	Ok(blog)
}

pub fn read(blogger: &Target, chamber: &Chamber) -> io::Result<Option<Target>> {
	let blogs = chamber.targets_with_property(BLOG_OWNER, &Arrow::Target(blogger.clone()))?;
	Ok(blogs.first().cloned())
}
