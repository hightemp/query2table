//! Files and images attached to a query: storage, parsing into cited fragments, and search.
//!
//! A place in a file is addressed like a web page, by an `attachment://` URL (see [`Locator`]),
//! so rows, research answers and links cite files through the same source plumbing as pages.

pub mod chunk;
pub mod model;
pub mod parse;
pub mod render;
pub mod retrieve;
pub mod store;
pub mod vision;
#[cfg(test)]
pub mod test_files;

pub use model::*;
