//! The AI agent that works the vault and examines the user (ADR 0009). Everything here is pure: it
//! builds the requests a provider understands, reads what comes back, builds the prompts and checks
//! what the model proposes. Sending a request is the job of a [`provider::Transport`] the application
//! supplies, so this crate does no I/O and links no network library.

pub mod assistant_backend;
pub mod exam;
pub mod exam_gen;
pub mod exam_grade;
pub mod lecture_budget;
pub mod lecture_grade;
pub mod lecture_groups;
pub mod lecture_peer;
pub mod lecture_prompt;
pub mod lecture_student;
pub mod lecture_task;
pub mod lecture_teammate;
pub mod mcp_client;
pub mod mcp_client_cfg;
pub mod mcp_client_chat;
pub mod mcp_route;
pub mod mcp_stage;
pub mod mcp_types;
pub mod mcp_uri;
pub mod model_catalog;
pub mod model_provider;
pub mod provider;
pub mod rubric_eval;
pub mod rubric_prompt;
pub mod settings;
pub mod sidecar_process;
pub mod sidecar_supervisor;
pub mod sidecar_types;
pub mod wiki;
pub mod wiki_change;
pub mod wiki_pages;
pub mod wire;
