//! Basic Usage:
//!
//! ```
//! use echo_lib::Echo;
//! let echo = Echo::connect("my-app", &std::env::temp_dir());
//! ```
extern crate rand;

pub use self::chamber::*;
pub use self::core::*;
pub use self::echo::Echo;
pub use self::object::*;

mod chamber;
mod core;
mod echo;
mod object;
pub mod util;
pub mod hamt;
pub mod diary;
pub mod bytes;

#[cfg(test)]
mod counter_tests;
