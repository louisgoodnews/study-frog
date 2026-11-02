//! `_stack.toml`: settings of one stack. Flat keys, all optional.

use serde::Deserialize;
use studyfrog_core::scheduler::scheduler_name;
use studyfrog_core::settings::StackSettings;

use crate::error::{ConfigError, from_toml};
use crate::values::{float_text, retention_value, scheduler_value};

/// File name of a stack's settings, next to its `_stack.md`.
pub const STACK_CONFIG_FILE: &str = "_stack.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStack {
    scheduler: Option<String>,
    request_retention: Option<f64>,
    daily_new_limit: Option<u32>,
    daily_review_limit: Option<u32>,
    familiarity: Option<bool>,
}

/// Parse and validate `_stack.toml`. Unknown keys are errors.
pub fn parse_stack_settings(text: &str) -> Result<StackSettings, ConfigError> {
    let raw: RawStack = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    Ok(StackSettings {
        scheduler: raw
            .scheduler
            .map(|n| scheduler_value(text, &n, "scheduler"))
            .transpose()?,
        request_retention: raw
            .request_retention
            .map(|r| retention_value(text, r, "request_retention"))
            .transpose()?,
        daily_new_limit: raw.daily_new_limit,
        daily_review_limit: raw.daily_review_limit,
        familiarity: raw.familiarity,
    })
}

const HEADER: &str = "# Settings for this stack. Every key is optional; omitted keys use the vault\n\
# defaults from studyfrog.toml.\n";

/// Render settings as a `_stack.toml` with a short header comment; unset keys are left out.
pub fn render_stack_settings(s: &StackSettings) -> String {
    let mut out = String::from(HEADER);
    if let Some(sched) = s.scheduler {
        out.push_str(&format!("scheduler = \"{}\"\n", scheduler_name(sched)));
    }
    if let Some(r) = s.request_retention {
        out.push_str(&format!("request_retention = {}\n", float_text(r)));
    }
    if let Some(n) = s.daily_new_limit {
        out.push_str(&format!("daily_new_limit = {n}\n"));
    }
    if let Some(n) = s.daily_review_limit {
        out.push_str(&format!("daily_review_limit = {n}\n"));
    }
    if let Some(on) = s.familiarity {
        out.push_str(&format!("familiarity = {on}\n"));
    }
    out
}
