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
	let blogger = blogger::read(&echo.chamber()?).unwrap().unwrap();
	let blog = blog::create_if_none(&blogger, &echo).unwrap();
	let posts = post::read_ordered(&blog, &chamber).unwrap();
	assert_eq!(posts.len(), 2);
	assert_eq!(chamber.string(&posts[0], post::BODY), "Elephant ears are big.");
	Ok(())
}

fn mutate(echo_name: &String) -> Result<(), Box<dyn Error>> {
	let echo = Echo::connect(&echo_name, &temp_dir());
	let blogger = blogger::create_if_none(&echo)?;
	let blog = blog::create_if_none(&blogger, &echo)?;
	post::create("Elephant ears", "Elephant ears are big.", &blog, &echo)?;
	let post = post::create("Kitten ears", "Kitten ears are cute.", &blog, &echo)?;
	assert!(echo.chamber()?.arrow_at_target_ring_or_none(&post, post::BLOG_ID).is_some());
	Ok(())
}

pub mod post;
pub mod blog;
pub mod blogger;
