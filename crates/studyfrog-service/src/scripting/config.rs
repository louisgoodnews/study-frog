//! `<app home>/scripts/scripts.toml`: the jobs the runner knows. Each `[[job]]` names a script in
//! the scripts folder and when it runs (a cron `schedule`, lifecycle events in `on`, or only by hand),
//! its timeout and what its control socket lets through. Invalid entries are reported and left out.

use serde::Deserialize;

use super::cron::{Cron, parse_cron};

/// The configuration file inside the scripts folder.
pub const CONFIG_FILE: &str = "scripts.toml";
/// Seconds a job may run when its entry names no timeout.
pub const DEFAULT_TIMEOUT_SECS: u64 = 60;
/// The longest timeout an entry may ask for (one hour).
pub const MAX_TIMEOUT_SECS: u64 = 3600;

/// What a script's control socket lets through (the per-script form of the control policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Access {
    /// Read and navigate only.
    Read,
    /// Also create, edit, move, rehearse, import and export.
    Write,
    /// Also delete (`allow_destructive = true`).
    Destructive,
}

/// The word for an access level.
pub fn access_name(a: Access) -> &'static str {
    match a {
        Access::Read => "read",
        Access::Write => "write",
        Access::Destructive => "destructive",
    }
}

/// One valid job.
#[derive(Debug, Clone, PartialEq)]
pub struct ScriptJob {
    /// Unique name.
    pub name: String,
    /// The script, relative to the scripts folder.
    pub script: String,
    /// Arguments after the script.
    pub args: Vec<String>,
    /// When it runs by the clock, and the text it was given as.
    pub schedule: Option<(Cron, String)>,
    /// Lifecycle events that start it.
    pub on: Vec<String>,
    /// Seconds before it is killed.
    pub timeout_secs: u64,
    /// What its socket lets through.
    pub access: Access,
    /// Switched on.
    pub enabled: bool,
}

/// The whole file: valid jobs, why the others were left out, and the Python to use.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScriptsConfig {
    /// Valid jobs, in file order.
    pub jobs: Vec<ScriptJob>,
    /// One line per entry (or the file) that could not be used.
    pub errors: Vec<String>,
    /// The interpreter for `.py` scripts (`python = "..."`, default `python3`).
    pub python: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    python: Option<String>,
    #[serde(default)]
    job: Vec<toml::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawJob {
    name: String,
    script: String,
    #[serde(default)]
    args: Vec<String>,
    schedule: Option<String>,
    #[serde(default)]
    on: Vec<String>,
    timeout: Option<u64>,
    access: Option<String>,
    #[serde(default)]
    allow_destructive: bool,
    enabled: Option<bool>,
}

/// A script path inside the scripts folder: relative, no `..`, not empty.
pub fn safe_script_path(path: &str) -> bool {
    let p = std::path::Path::new(path);
    !path.trim().is_empty()
        && p.is_relative()
        && p.components()
            .all(|c| matches!(c, std::path::Component::Normal(_)))
}

fn access_of(raw: &RawJob) -> Result<Access, String> {
    let base = match raw.access.as_deref().unwrap_or("read") {
        "read" => Access::Read,
        "write" => Access::Write,
        other => return Err(format!("access = \"{other}\" (read or write)")),
    };
    Ok(if raw.allow_destructive {
        Access::Destructive
    } else {
        base
    })
}

fn job_of(raw: RawJob, events: &[&str]) -> Result<ScriptJob, String> {
    if raw.name.trim().is_empty() {
        return Err("a job needs a name".into());
    }
    if !safe_script_path(&raw.script) {
        return Err(format!(
            "script = \"{}\" must be a path inside the scripts folder",
            raw.script
        ));
    }
    if let Some(bad) = raw.on.iter().find(|e| !events.contains(&e.as_str())) {
        return Err(format!(
            "unknown event {bad} (one of {})",
            events.join(", ")
        ));
    }
    let timeout_secs = raw.timeout.unwrap_or(DEFAULT_TIMEOUT_SECS);
    if !(1..=MAX_TIMEOUT_SECS).contains(&timeout_secs) {
        return Err(format!(
            "timeout = {timeout_secs} (1 to {MAX_TIMEOUT_SECS} seconds)"
        ));
    }
    let schedule = match &raw.schedule {
        Some(text) => Some((parse_cron(text)?, text.clone())),
        None => None,
    };
    Ok(ScriptJob {
        access: access_of(&raw)?,
        name: raw.name,
        script: raw.script,
        args: raw.args,
        schedule,
        on: raw.on,
        timeout_secs,
        enabled: raw.enabled.unwrap_or(true),
    })
}

/// Read the file's text; `events` are the lifecycle events a job may name.
pub fn parse_scripts_toml(text: &str, events: &[&str]) -> ScriptsConfig {
    let mut cfg = ScriptsConfig {
        python: "python3".into(),
        ..ScriptsConfig::default()
    };
    let file: RawFile = match toml::from_str(text) {
        Ok(f) => f,
        Err(e) => {
            cfg.errors.push(format!("{CONFIG_FILE}: {e}"));
            return cfg;
        }
    };
    cfg.python = file.python.unwrap_or(cfg.python);
    for (i, value) in file.job.into_iter().enumerate() {
        let job = RawJob::deserialize(value)
            .map_err(|e| e.to_string())
            .and_then(|raw| job_of(raw, events));
        match job {
            Ok(j) if cfg.jobs.iter().any(|o| o.name == j.name) => {
                cfg.errors
                    .push(format!("job {}: the name {} is used twice", i + 1, j.name))
            }
            Ok(j) => cfg.jobs.push(j),
            Err(e) => cfg.errors.push(format!("job {}: {e}", i + 1)),
        }
    }
    cfg
}
