//! Extension points as plain data. An extension is a crate that depends on this one and offers a
//! `register` function; the application calls it at start-up. Nothing in the core crates changes.
//!
//! There are two extension points:
//! - an [`Importer`] turns the text of a file into cards;
//! - a [`CardType`] turns one piece of source text into one or more cards (a cloze note becomes a
//!   card per gap, for example).
//!
//! Both are records of function pointers, not trait objects: they are values that can be listed,
//! compared and tested like any other data, and a [`Registry`] is a plain `Vec` of them.

pub mod registry;
pub mod types;
