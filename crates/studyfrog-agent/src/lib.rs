//! The AI agent that works the vault and examines the user (ADR 0009). Everything here is pure: it
//! builds the requests a provider understands, reads what comes back, builds the prompts and checks
//! what the model proposes. Sending a request is the job of a [`provider::Transport`] the application
//! supplies, so this crate does no I/O and links no network library.

pub mod exam;
pub mod mcp_route;
pub mod mcp_stage;
pub mod mcp_types;
pub mod mcp_uri;
pub mod provider;
pub mod settings;
pub mod wiki;
pub mod wiki_change;
pub mod wiki_pages;
pub mod wire;
