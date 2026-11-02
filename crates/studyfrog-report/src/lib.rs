//! Reporting and forecasting: retention by interval, answer latency, projected review load, and a
//! backtest of that projection. All pure; the service layer reads the history and calls these.

pub mod backtest;
pub mod explore;
pub mod familiarity;
mod familiarity_types;
pub mod forecast;
pub mod latency;
pub mod replay;
pub mod retention;
pub mod simulate;
