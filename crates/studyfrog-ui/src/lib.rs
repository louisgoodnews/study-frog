//! GUI layer. Thin: renders state and dispatches to the domain crates; contains no business logic.
//!
//! Structure (ADR 0002): `state`, `msg`, `effect` and `update` are the pure core and know nothing
//! about egui; `exec` performs effects; `view` and `model` draw; `runtime` and `app` are the glue.

pub mod app;
pub mod browse;
pub mod effect;
pub mod exec;
pub mod exec_run;
pub mod model;
pub mod msg;
pub mod run_state;
pub mod run_update;
pub mod runtime;
pub mod setup;
pub mod setup_form;
pub mod state;
pub mod summary;
pub mod update;
pub mod view;

pub use app::run;
