//! `achievements.toml` (vault root, optional): the achievements of this vault, replacing the built-in set.

use std::collections::HashSet;

use serde::Deserialize;
use studyfrog_core::achievement::{Rule, metric_from_name, rule_problem, scope_from_name};

use crate::error::{ConfigError, config_error, from_toml, line_of_key};

/// File name in the vault root.
pub const ACHIEVEMENTS_FILE: &str = "achievements.toml";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    achievement: Vec<RawRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    id: String,
    title: String,
    #[serde(default)]
    description: String,
    scope: String,
    metric: String,
    threshold: u32,
}

fn rule_of(text: &str, raw: RawRule) -> Result<Rule, ConfigError> {
    let at = |key: &str| line_of_key(text, key);
    let scope = scope_from_name(&raw.scope).ok_or_else(|| {
        config_error(
            at("scope"),
            format!(
                "scope must be \"global\" or \"stack\", not \"{}\"",
                raw.scope
            ),
        )
    })?;
    let metric = metric_from_name(&raw.metric)
        .ok_or_else(|| config_error(at("metric"), format!("unknown metric \"{}\"", raw.metric)))?;
    let rule = Rule {
        id: raw.id,
        title: raw.title,
        description: raw.description,
        scope,
        metric,
        threshold: raw.threshold,
    };
    match rule_problem(&rule) {
        Some(why) => Err(config_error(
            at("id"),
            format!("achievement \"{}\": {why}", rule.id),
        )),
        None => Ok(rule),
    }
}

/// Parse `achievements.toml`. Unknown keys, unknown metrics, unusable rules and repeated ids are errors.
pub fn parse_achievements(text: &str) -> Result<Vec<Rule>, ConfigError> {
    let raw: RawFile = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    let rules = raw
        .achievement
        .into_iter()
        .map(|r| rule_of(text, r))
        .collect::<Result<Vec<_>, _>>()?;
    let mut seen = HashSet::new();
    match rules.iter().find(|r| !seen.insert(r.id.as_str())) {
        Some(dup) => Err(config_error(
            None,
            format!("achievement id \"{}\" is used twice", dup.id),
        )),
        None => Ok(rules),
    }
}
