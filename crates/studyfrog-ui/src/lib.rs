//! GUI layer. Thin: renders state and dispatches to the domain crates; contains no business logic.
//!
//! Structure (ADR 0002): `state`, `msg`, `effect` and `update` are the pure core and know nothing
//! about egui; `exec` performs effects; `view` and `model` draw; `runtime` and `app` are the glue.

pub mod app;
pub mod browse;
pub mod edit_form;
pub mod editor;
pub mod effect;
pub mod exec;
pub mod exec_plan;
pub mod exec_run;
pub mod graph;
pub mod kanban;
pub mod kanban_msg;
pub mod model;
pub mod msg;
pub mod plan_msg;
pub mod planner;
pub mod review;
pub mod run_state;
pub mod run_update;
pub mod runtime;
pub mod session_form;
pub mod setup;
pub mod setup_form;
pub mod state;
pub mod summary;
pub mod update;
pub mod view;

pub use app::run;
