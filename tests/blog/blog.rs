use std::io;

use recurvedb::{Arrow, Bundle, Recurve, Ring, Target};

pub const BLOG_TITLE: &Ring = &Ring::Static { aspect: "Blog", name: "title" };
pub const BLOG_OWNER: &Ring = &Ring::Static { aspect: "Blog", name: "owner" };

pub fn create_if_none(blogger: &Target, recurve: &Recurve) -> io::Result<Target> {
	let old_blog = read(blogger, &recurve.to_bundle()?)?;
	let blog = match old_blog {
		Some(target) => target.clone(),
		None => recurve.draw(|write| {
			let blog = write.new_target("blog");
			write.release_target_properties(&blog, vec![
				(BLOG_OWNER, Arrow::Target(blogger.to_owned())),
				(BLOG_TITLE, Arrow::String("Musings".to_string())),
			]);
			blog
		})?,
	};
	Ok(blog)
}

pub fn read(blogger: &Target, bundle: &Bundle) -> io::Result<Option<Target>> {
	let blogs = bundle.targets_with_property(BLOG_OWNER, &Arrow::Target(blogger.clone()))?;
	Ok(blogs.first().cloned())
}
