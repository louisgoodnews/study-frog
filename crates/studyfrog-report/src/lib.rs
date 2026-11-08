//! Reporting and forecasting: retention by interval, answer latency, projected review load, and a
//! backtest of that projection. All pure; the service layer reads the history and calls these.

pub mod backtest;
pub mod calibration;
pub mod calibration_types;
pub mod chronotype;
pub mod cmrr;
pub mod confusion;
pub mod exam_pacer;
pub mod explore;
pub mod familiarity;
mod familiarity_types;
pub mod fatigue;
pub mod forecast;
pub mod forecast_sim;
pub mod latency;
pub mod leech;
pub mod replay;
pub mod retention;
pub mod retention_survival;
pub mod session_log;
pub mod sim_engine;
mod sim_types;
pub mod simulate;
mod survival_types;
