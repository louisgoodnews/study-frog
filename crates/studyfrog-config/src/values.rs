//! Shared value helpers: float text and validation.

use studyfrog_core::scheduler::{Scheduler, scheduler_from_name};

use crate::error::{ConfigError, config_error, line_of_key};

/// Shortest decimal text of an `f32` that is also a valid TOML float.
pub fn float_text(x: f32) -> String {
    let text = x.to_string();
    if text.contains(['.', 'e', 'E']) {
        text
    } else {
        format!("{text}.0")
    }
}

/// A scheduler name, or an error pointing at `key`.
pub fn scheduler_value(text: &str, name: &str, key: &str) -> Result<Scheduler, ConfigError> {
    scheduler_from_name(name).ok_or_else(|| {
        config_error(
            line_of_key(text, key),
            format!("{key} must be \"fsrs\" or \"sm2\""),
        )
    })
}

/// A retention strictly between 0 and 1, or an error pointing at `key`.
pub fn retention_value(text: &str, value: f64, key: &str) -> Result<f32, ConfigError> {
    if value > 0.0 && value < 1.0 {
        Ok(value as f32)
    } else {
        Err(config_error(
            line_of_key(text, key),
            format!("{key} must be between 0 and 1 (exclusive)"),
        ))
    }
}
