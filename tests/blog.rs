use std::env::temp_dir;
use std::io;

use echo_lib::{Chamber, Echo, ObjectId, Point, Target};
use echo_lib::util::unique_name;

#[test]
fn it_works() {
	let echo = Echo::connect(&unique_name("blog-test"), &temp_dir());
	let blogger_id = init_blogger(&echo).unwrap();
	let blog_id = init_blog(&blogger_id, &echo).unwrap();
	add_post("Elephant ears", "Elephant ears are big.", &blog_id, &echo).unwrap();
	add_post("Kitten ears", "Kitten ears are cute.", &blog_id, &echo).unwrap();
	let chamber = echo.chamber().unwrap();
	let mut posts = ordered_posts(&blog_id, &chamber).unwrap();
	assert_eq!(posts.len(), 2);
	let kitten_post = posts.pop().unwrap();
	assert_eq!(chamber.string(&kitten_post, POST_BODY), "Kitten ears are cute.");
}

fn ordered_posts(blog_id: &ObjectId, chamber: &Chamber) -> io::Result<Vec<ObjectId>> {
	let mut posts = chamber.objects_with_property(POST_BLOG, &Target::Object(blog_id.to_owned()))?;
	posts.sort_by_key(|it| chamber.string(it, POST_TITLE));
	Ok(posts)
}

fn add_post(title: &str, body: &str, blog_id: &ObjectId, echo: &Echo) -> io::Result<ObjectId> {
	echo.write(|write| {
		let post_id = write.new_object_id("blog-post");
		write.object_attributes(&post_id, vec![
			(POST_TITLE, Target::String(title.to_string())),
			(POST_BODY, Target::String(body.to_string())),
			(POST_BLOG, Target::Object(blog_id.to_owned())),
		]);
		post_id
	})
}

fn init_blog(blogger_id: &ObjectId, echo: &Echo) -> io::Result<ObjectId> {
	let chamber = echo.chamber()?;
	let blogs = chamber.objects_with_point(BLOG_OWNER)?;
	let blog_id = match blogs.first() {
		Some(id) => id.clone(),
		None => echo.write(|write| {
			let blog_id = write.new_object_id("blog");
			write.object_attributes(&blog_id, vec![
				(BLOG_OWNER, Target::Object(blogger_id.to_owned())),
				(BLOG_TITLE, Target::String("Musings".to_string()))
			]);
			blog_id
		})?,
	};
	Ok(blog_id)
}

fn init_blogger(echo: &Echo) -> io::Result<ObjectId> {
	let chamber = echo.chamber()?;
	let bloggers = chamber.objects_with_point(BLOGGER_NAME)?;
	let blogger_id = match bloggers.first() {
		Some(id) => id.clone(),
		None => echo.write(|write| {
			let blogger_id = write.new_object_id("blogger");
			write.object_attributes(&blogger_id, vec![
				(BLOGGER_NAME, Target::String("Alice".to_string()))
			]);
			blogger_id
		})?,
	};
	Ok(blogger_id)
}

pub const POST_TITLE: &Point = &Point::Static { aspect: "BlogPost", name: "title" };
pub const POST_BODY: &Point = &Point::Static { aspect: "BlogPost", name: "body" };
pub const POST_BLOG: &Point = &Point::Static { aspect: "BlogPost", name: "blog" };
pub const BLOG_TITLE: &Point = &Point::Static { aspect: "Blog", name: "title" };
pub const BLOG_OWNER: &Point = &Point::Static { aspect: "Blog", name: "owner" };
pub const BLOGGER_NAME: &Point = &Point::Static { aspect: "Blogger", name: "name" };
