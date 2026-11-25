//! The local setup assistant (STORY-0206): the hardware audit, the state of the local Ollama
//! service, the models that fit this computer, and the setup guide. The parsers and the fit rule are
//! pure; the probes read files and run commands with a timeout; the HTTP probe is supplied by the
//! host, so this crate links no network library.

pub mod audit_parse;
pub mod audit_probe;
pub mod audit_types;
pub mod catalog;
pub mod client;
pub mod fit;
pub mod guide;
pub mod pull;
pub mod run;
