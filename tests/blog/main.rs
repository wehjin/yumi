use std::env::temp_dir;
use std::error::Error;

use echodb::Echo;
use echodb::util::unique_name;

#[test]
fn it_works() -> Result<(), Box<dyn Error>> {
	let echo_name = unique_name("blog-test");
	mutate(&echo_name)?;
	review(&echo_name)?;
	Ok(())
}

fn review(echo_name: &String) -> Result<(), Box<dyn Error>> {
	let echo = Echo::connect(&echo_name, &temp_dir());
	let chamber = echo.chamber().unwrap();
	let blogger_id = blogger::read(&echo.chamber()?).unwrap().unwrap();
	let blog_id = blog::create_if_none(&blogger_id, &echo).unwrap();
	let posts = post::read_ordered(&blog_id, &chamber).unwrap();
	assert_eq!(posts.len(), 2);
	assert_eq!(chamber.string(&posts[0], post::BODY), "Elephant ears are big.");
	Ok(())
}

fn mutate(echo_name: &String) -> Result<(), Box<dyn Error>> {
	let echo = Echo::connect(&echo_name, &temp_dir());
	let blogger_id = blogger::create_if_none(&echo)?;
	let blog_id = blog::create_if_none(&blogger_id, &echo)?;
	post::create("Elephant ears", "Elephant ears are big.", &blog_id, &echo)?;
	let post_id = post::create("Kitten ears", "Kitten ears are cute.", &blog_id, &echo)?;
	assert!(echo.chamber()?.target_at_object_point_or_none(&post_id, post::BLOG_ID).is_some());
	Ok(())
}

pub mod post;
pub mod blog;
pub mod blogger;
