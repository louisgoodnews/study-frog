//! Scripts run out of process (ADR 0013): the `scripts.toml` configuration, cron schedules, which
//! jobs are due or bound to an event, running one as a child process, and logging how it went.

pub mod config;
pub mod cron;
pub mod due;
pub mod exec;
pub mod log;
