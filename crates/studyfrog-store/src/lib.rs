//! Persistence: SQLite for structured data and a plain-file layer for Markdown content. The only crate that performs storage I/O.

pub mod achievements;
pub mod app_config;
pub mod board_store;
pub mod db;
pub mod error;
pub mod hash;
pub mod index;
pub mod items;
pub mod library;
pub mod link_graph;
pub mod migrate;
pub mod overview;
pub mod poll;
pub mod reindex;
pub mod reports;
pub mod run_load;
pub mod runs;
pub mod scan;
pub mod schedule;
pub mod schema;
pub mod sessions;
pub mod settings;
pub mod stackcfg;
pub mod stackmap;
pub mod vault;
pub mod vault_config;
