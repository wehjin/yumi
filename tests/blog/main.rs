use std::env::temp_dir;
use std::error::Error;

use recurvedb::Recurve;
use recurvedb::util::unique_name;

#[test]
fn it_works() -> Result<(), Box<dyn Error>> {
	let db_name = unique_name("blog-test");
	mutate(&db_name)?;
	review(&db_name)?;
	Ok(())
}

fn review(db_name: &String) -> Result<(), Box<dyn Error>> {
	let recurve = Recurve::connect(&db_name, &temp_dir());
	let chamber = recurve.chamber().unwrap();
	let blogger = blogger::read(&recurve.chamber()?).unwrap().unwrap();
	let blog = blog::create_if_none(&blogger, &recurve).unwrap();
	let posts = post::read_ordered(&blog, &chamber).unwrap();
	assert_eq!(posts.len(), 2);
	assert_eq!(chamber.string(&posts[0], post::BODY), "Elephant ears are big.");
	Ok(())
}

fn mutate(db_name: &String) -> Result<(), Box<dyn Error>> {
	let recurve = Recurve::connect(&db_name, &temp_dir());
	let blogger = blogger::create_if_none(&recurve)?;
	let blog = blog::create_if_none(&blogger, &recurve)?;
	post::create("Elephant ears", "Elephant ears are big.", &blog, &recurve)?;
	let post = post::create("Kitten ears", "Kitten ears are cute.", &blog, &recurve)?;
	assert!(recurve.chamber()?.arrow_at_target_ring_or_none(&post, post::BLOG_ID).is_some());
	Ok(())
}

pub mod post;
pub mod blog;
pub mod blogger;
