//! Markdown and front-matter parsing/rendering for all content kinds. Pure transforms from text to core types and back.

pub mod build;
pub mod codeblock;
pub mod doc;
pub mod error;
pub mod fence;
pub mod links;
pub mod parse;
pub mod raw;
pub mod render;
pub mod rfc3339;
pub mod sections;
pub mod ulid;
pub mod validate;
