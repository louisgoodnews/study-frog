//! GUI layer. Thin: renders state and dispatches to the domain crates; contains no business logic.
//!
//! Structure (ADR 0002): `state`, `msg`, `effect` and `update` are the pure core and know nothing
//! about egui; `exec` performs effects; `view` and `model` draw; `runtime` and `app` are the glue.

pub mod app;
pub mod effect;
pub mod exec;
pub mod model;
pub mod msg;
pub mod runtime;
pub mod state;
pub mod update;
pub mod view;

pub use app::run;
