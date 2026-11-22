//! The scheduled and event-bound jobs of `scripts.toml`, for the app's job editor (STORY-0151): each job with the time
//! it next runs, the file written back after a check, the job's history in the app log, and the scripts that can be
//! run by hand. The clock is passed in; the file is read and written here.

use std::fs;
use std::path::Path;

use serde::Serialize;
use studyfrog_core::civil::{LocalTime, add_days, format_date, utc_to_local};
use studyfrog_core::time::UnixMs;

use crate::logging::tail_lines;
use crate::scripting::config::{Access, CONFIG_FILE, ScriptsConfig, parse_scripts_toml};
use crate::scripting::cron::{Cron, cron_matches};
use studyfrog_config::paths::LOG_TEXT_FILE;

/// One job as the editor shows and changes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobRow {
    /// Unique name.
    pub name: String,
    /// The script, relative to the scripts folder.
    pub script: String,
    /// Arguments after the script.
    pub args: Vec<String>,
    /// The schedule as written (empty: none).
    pub schedule: String,
    /// The lifecycle events that start it.
    pub on: Vec<String>,
    /// Seconds before it is killed.
    pub timeout_secs: u64,
    /// What its socket lets through.
    pub access: Access,
    /// Switched on.
    pub enabled: bool,
    /// When it next runs by the clock (`None`: no schedule, or switched off, or none within a year).
    pub next_run: Option<String>,
}

/// The highest number of minutes looked at for a next run: one year.
const LOOK_AHEAD_MINUTES: u32 = 366 * 24 * 60;

/// The first minute after `from` at which `c` fires, looking at most a year ahead.
pub fn next_run(c: &Cron, from: LocalTime) -> Option<LocalTime> {
    let mut t = from;
    for _ in 0..LOOK_AHEAD_MINUTES {
        t = advance_minute(t);
        if cron_matches(c, t) {
            return Some(t);
        }
    }
    None
}

fn advance_minute(t: LocalTime) -> LocalTime {
    if t.minute + 1 >= 24 * 60 {
        LocalTime {
            date: add_days(t.date, 1),
            minute: 0,
        }
    } else {
        LocalTime {
            date: t.date,
            minute: t.minute + 1,
        }
    }
}

/// `2027-03-01 08:00`.
pub fn format_when(t: LocalTime) -> String {
    format!(
        "{} {:02}:{:02}",
        format_date(t.date),
        t.minute / 60,
        t.minute % 60
    )
}

/// The jobs of a read configuration, each with its next run as seen at `now` in the zone of `offset_minutes`.
pub fn rows_of(cfg: &ScriptsConfig, now: UnixMs, offset_minutes: i32) -> Vec<JobRow> {
    let here = utc_to_local(now, offset_minutes);
    cfg.jobs
        .iter()
        .map(|j| JobRow {
            name: j.name.clone(),
            script: j.script.clone(),
            args: j.args.clone(),
            schedule: j
                .schedule
                .as_ref()
                .map(|(_, text)| text.clone())
                .unwrap_or_default(),
            on: j.on.clone(),
            timeout_secs: j.timeout_secs,
            access: j.access,
            enabled: j.enabled,
            next_run: j
                .schedule
                .as_ref()
                .filter(|_| j.enabled)
                .and_then(|(c, _)| next_run(c, here))
                .map(format_when),
        })
        .collect()
}

/// The configuration as the file has it now (errors included).
pub fn read_config(scripts: &Path, events: &[&str]) -> ScriptsConfig {
    match fs::read_to_string(scripts.join(CONFIG_FILE)) {
        Ok(text) => parse_scripts_toml(&text, events),
        Err(_) => parse_scripts_toml("", events),
    }
}

#[derive(Serialize)]
struct FileOut<'a> {
    python: &'a str,
    job: Vec<JobOut>,
}

#[derive(Serialize)]
struct JobOut {
    name: String,
    script: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schedule: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    on: Vec<String>,
    timeout: u64,
    access: &'static str,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    allow_destructive: bool,
    enabled: bool,
}

/// The file's text for these jobs. A destructive job is written as write access plus `allow_destructive`.
pub fn to_text(rows: &[JobRow], python: &str) -> Result<String, String> {
    let out = FileOut {
        python,
        job: rows
            .iter()
            .map(|r| JobOut {
                name: r.name.clone(),
                script: r.script.clone(),
                args: r.args.clone(),
                schedule: (!r.schedule.trim().is_empty()).then(|| r.schedule.trim().to_owned()),
                on: r.on.clone(),
                timeout: r.timeout_secs,
                access: match r.access {
                    Access::Read => "read",
                    Access::Write | Access::Destructive => "write",
                },
                allow_destructive: r.access == Access::Destructive,
                enabled: r.enabled,
            })
            .collect(),
    };
    toml::to_string(&out).map_err(|e| e.to_string())
}

/// Write the jobs to `scripts.toml`, but only when the text reads back with no errors (the file is replaced
/// whole, through a temporary file, so a failed write leaves the old one).
pub fn save(scripts: &Path, rows: &[JobRow], python: &str, events: &[&str]) -> Result<(), String> {
    let text = to_text(rows, python)?;
    let check = parse_scripts_toml(&text, events);
    if !check.errors.is_empty() {
        return Err(check.errors.join("; "));
    }
    fs::create_dir_all(scripts).map_err(|e| e.to_string())?;
    let target = scripts.join(CONFIG_FILE);
    let tmp = scripts.join(format!("{CONFIG_FILE}.tmp"));
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &target).map_err(|e| e.to_string())
}

/// The last `n` lines of the app log about this job's runs (their outcome lines name the job).
pub fn history(logs: &Path, job: &str, n: usize) -> Vec<String> {
    let needle = format!("script {job} ");
    let lines = tail_lines(&logs.join(LOG_TEXT_FILE), 2000).unwrap_or_default();
    let mut found: Vec<String> = lines.into_iter().filter(|l| l.contains(&needle)).collect();
    let keep = found.len().saturating_sub(n);
    found.drain(..keep);
    found
}

/// The script files in the scripts folder that could be run by hand, sorted.
pub fn scripts_available(scripts: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(scripts) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n != CONFIG_FILE && crate::scripting::config::safe_script_path(n))
        .collect();
    names.sort();
    names
}
