//! The runner in every instance (ADR 0013): scheduled jobs each minute their schedule matches, jobs
//! bound to lifecycle events, and runs asked for by name. Each run happens on a thread of its own; a
//! job still running is not started again.

use std::collections::HashSet;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use studyfrog_config::paths::AppPaths;
use studyfrog_core::time::UnixMs;
use studyfrog_service::scripting::config::{
    Access, CONFIG_FILE, DEFAULT_TIMEOUT_SECS, ScriptJob, ScriptsConfig, access_name,
    parse_scripts_toml,
};
use studyfrog_service::scripting::due::{Ticker, find_job, jobs_for_event, tick};
use studyfrog_service::timezone::local_offset_minutes;

use crate::control_conn::ConnCtx;
use crate::control_events::{EventBus, publish, subscribe};
use crate::script_events::EVENT_KINDS;
use crate::script_launch::{Launch, Route, finished_event, launch};

/// What the runner works with. Cloned into each of its threads.
#[derive(Clone)]
pub struct Runner {
    /// The application's directories (`scripts/`, `logs/`, `run/`).
    pub paths: AppPaths,
    /// Requests of the scripts go into this instance.
    pub ctx: ConnCtx,
    /// Lifecycle events: listened to, and `script_finished` published.
    pub bus: EventBus,
    /// Jobs running now.
    pub running: Arc<Mutex<HashSet<String>>>,
    /// The pause between looks at a child process.
    pub pause: fn(),
}

/// `scripts.toml` in `scripts`; no file is no jobs.
pub fn load_config(scripts: &Path) -> ScriptsConfig {
    match std::fs::read_to_string(scripts.join(CONFIG_FILE)) {
        Ok(text) => parse_scripts_toml(&text, EVENT_KINDS),
        Err(_) => parse_scripts_toml("", EVENT_KINDS),
    }
}

/// The pause between looks at a running script (20 ms).
pub fn child_pause() {
    let (_tx, rx) = std::sync::mpsc::channel::<()>();
    let _ = rx.recv_timeout(std::time::Duration::from_millis(20));
}

/// The wall clock in Unix milliseconds.
pub fn wall_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(0))
}

/// The run of a configured job.
pub fn launch_of(job: &ScriptJob, event: Option<Value>) -> Launch {
    Launch {
        job: job.name.clone(),
        script: job.script.clone(),
        args: job.args.clone(),
        access: job.access,
        timeout_secs: job.timeout_secs,
        event,
    }
}

/// Start `l` on a thread of its own unless a run of the same job is going on; whether it started.
pub fn start(r: &Runner, python: &str, l: Launch) -> bool {
    let Ok(mut running) = r.running.lock() else {
        return false;
    };
    if !running.insert(l.job.clone()) {
        return false;
    }
    drop(running);
    let (r, python) = (r.clone(), python.to_owned());
    std::thread::spawn(move || {
        let route = Route::Instance(r.ctx.clone());
        let o = launch(&r.paths, &python, &l, &route, &r.pause, wall_ms());
        if let Ok(mut running) = r.running.lock() {
            running.remove(&l.job);
        }
        publish(&r.bus, &[finished_event(&l.job, &o)], wall_ms());
    });
    true
}

/// Start the job named `what` (or whose script is `what`), else the script file `what` in the scripts
/// folder with write access; what was started, or why not.
pub fn run_named(r: &Runner, what: &str, args: &[String]) -> Result<String, String> {
    let cfg = load_config(&r.paths.scripts);
    let l = match find_job(&cfg, what) {
        Some(job) => Launch {
            args: if args.is_empty() {
                job.args.clone()
            } else {
                args.to_vec()
            },
            ..launch_of(job, None)
        },
        None if r.paths.scripts.join(what).is_file()
            && studyfrog_service::scripting::config::safe_script_path(what) =>
        {
            Launch {
                job: what.to_owned(),
                script: what.to_owned(),
                args: args.to_vec(),
                access: Access::Write,
                timeout_secs: DEFAULT_TIMEOUT_SECS,
                event: None,
            }
        }
        None => {
            return Err(format!(
                "no job or script {what} in {}",
                r.paths.scripts.display()
            ));
        }
    };
    let name = l.job.clone();
    if start(r, &cfg.python, l) {
        Ok(name)
    } else {
        Err(format!("{name} is still running"))
    }
}

/// One look at the clock: start the jobs due since the last look. Returns their names.
pub fn scheduler_step(
    r: &Runner,
    ticker: &mut Ticker,
    now_ms: i64,
    offset_minutes: i32,
) -> Vec<String> {
    if ticker.last_minute == Some(now_ms.div_euclid(60_000)) {
        return Vec::new();
    }
    let cfg = load_config(&r.paths.scripts);
    let names = tick(ticker, &cfg, now_ms, offset_minutes);
    for job in cfg.jobs.iter().filter(|j| names.contains(&j.name)) {
        start(r, &cfg.python, launch_of(job, None));
    }
    names
}

/// The jobs an event starts (not the job whose own end the event reports). Returns their names.
pub fn event_step(r: &Runner, event: &Value) -> Vec<String> {
    let kind = event.get("kind").and_then(Value::as_str).unwrap_or("");
    let own = event.pointer("/data/job").and_then(Value::as_str);
    let cfg = load_config(&r.paths.scripts);
    jobs_for_event(&cfg, kind)
        .into_iter()
        .filter(|j| Some(j.name.as_str()) != own)
        .filter(|j| start(r, &cfg.python, launch_of(j, Some(event.clone()))))
        .map(|j| j.name.clone())
        .collect()
}

/// Run the scheduler (looking every `wait`) and the event listener on threads of their own.
pub fn start_runner(r: &Runner, wait: fn()) {
    let events = subscribe(&r.bus);
    let er = r.clone();
    std::thread::spawn(move || {
        for event in events.iter() {
            event_step(&er, &event);
        }
    });
    let sr = r.clone();
    std::thread::spawn(move || {
        let mut ticker = Ticker::default();
        loop {
            let now = wall_ms();
            scheduler_step(&sr, &mut ticker, now, local_offset_minutes(UnixMs(now)));
            wait();
        }
    });
}

/// The jobs of `scripts.toml`, one per line, then what is wrong with the file.
pub fn list_text(p: &AppPaths) -> (String, bool) {
    let cfg = load_config(&p.scripts);
    let mut text = format!("{}\n", p.scripts.join("scripts.toml").display());
    for j in &cfg.jobs {
        let when = j
            .schedule
            .as_ref()
            .map(|(_, t)| format!("schedule \"{t}\""))
            .into_iter();
        let when: Vec<String> = when
            .chain((!j.on.is_empty()).then(|| format!("on {}", j.on.join(","))))
            .collect();
        let when = if when.is_empty() {
            "by hand".to_owned()
        } else {
            when.join(", ")
        };
        let off = if j.enabled { "" } else { " (off)" };
        text += &format!(
            "{}{off}: {} [{when}; {}; {} s]\n",
            j.name,
            j.script,
            access_name(j.access),
            j.timeout_secs
        );
    }
    for e in &cfg.errors {
        text += &format!("error: {e}\n");
    }
    (text, cfg.errors.is_empty())
}
