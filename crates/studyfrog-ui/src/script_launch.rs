//! One run of a script (ADR 0013): its own control socket with the job's access, the command line and
//! environment, the child process with its timeout, the log entries and the `script_finished` event.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value, json};
use studyfrog_config::paths::AppPaths;
use studyfrog_service::scripting::config::{Access, access_name};
use studyfrog_service::scripting::exec::{ScriptOutcome, ScriptRun, run_script, succeeded};
use studyfrog_service::scripting::log::{LOG_OUTPUT_CHARS, log_outcome, tail_chars};

use crate::control_cmd::ControlPolicy;
use crate::control_conn::ConnCtx;
use crate::script_client::{install_client, lib_dir};
use crate::script_events::Lifecycle;

/// Where a run's requests go.
#[derive(Clone)]
pub enum Route {
    /// Into this instance, under the job's access.
    Instance(ConnCtx),
    /// To the socket of another running instance, through a proxy that refuses what the access does not allow.
    Proxy(PathBuf),
    /// To a socket that already enforces the access (a headless instance started for the run).
    Direct(PathBuf),
}

/// What to run.
#[derive(Debug, Clone, PartialEq)]
pub struct Launch {
    /// The job's name (the script's file name for an ad-hoc run).
    pub job: String,
    /// The script: relative to the scripts folder, or absolute.
    pub script: String,
    /// Its arguments.
    pub args: Vec<String>,
    /// What its socket lets through.
    pub access: Access,
    /// Seconds before it is killed.
    pub timeout_secs: u64,
    /// The event that started it, as subscribers get it.
    pub event: Option<Value>,
}

/// The control policy for an access level.
pub fn access_policy(a: Access) -> ControlPolicy {
    ControlPolicy {
        allow_write: a >= Access::Write,
        allow_destructive: a == Access::Destructive,
    }
}

/// The program, arguments and environment of a run: `.py` scripts run with `python`, others are
/// started themselves (they must be executable).
pub fn command_for(paths: &AppPaths, python: &str, l: &Launch, socket: &Path) -> ScriptRun {
    let script = paths.scripts.join(&l.script);
    let path = script.display().to_string();
    let (program, mut args) = if l.script.ends_with(".py") {
        (python.to_owned(), vec![path])
    } else {
        (path, Vec::new())
    };
    args.extend(l.args.iter().cloned());
    let lib = lib_dir(&paths.scripts).display().to_string();
    let pythonpath = match std::env::var("PYTHONPATH") {
        Ok(old) if !old.is_empty() => format!("{lib}:{old}"),
        _ => lib,
    };
    let mut env = vec![
        ("STUDYFROG_SOCKET".to_owned(), socket.display().to_string()),
        (
            "STUDYFROG_HOME".to_owned(),
            paths.root.display().to_string(),
        ),
        ("STUDYFROG_JOB".to_owned(), l.job.clone()),
        (
            "STUDYFROG_ACCESS".to_owned(),
            access_name(l.access).to_owned(),
        ),
        ("PYTHONPATH".to_owned(), pythonpath),
        ("PYTHONUNBUFFERED".to_owned(), "1".to_owned()),
    ];
    env.extend(
        l.event
            .as_ref()
            .map(|e| ("STUDYFROG_EVENT".to_owned(), e.to_string())),
    );
    env.extend(
        std::env::current_exe()
            .ok()
            .map(|p| ("STUDYFROG_BIN".to_owned(), p.display().to_string())),
    );
    ScriptRun {
        program,
        args,
        cwd: paths.scripts.clone(),
        env,
        timeout: Duration::from_secs(l.timeout_secs),
    }
}

/// The `script_finished` event of a run.
pub fn finished_event(job: &str, o: &ScriptOutcome) -> Lifecycle {
    Lifecycle {
        kind: "script_finished",
        data: json!({
            "job": job, "ok": succeeded(o), "code": o.code, "timed_out": o.timed_out,
            "error": o.spawn_error, "millis": o.millis,
            "stdout": tail_chars(&o.stdout, LOG_OUTPUT_CHARS), "stderr": tail_chars(&o.stderr, LOG_OUTPUT_CHARS),
        }),
    }
}

fn socket_for(
    paths: &AppPaths,
    route: &Route,
    a: Access,
) -> std::io::Result<Option<crate::script_socket::RunSocket>> {
    use crate::script_socket::{serve_proxy, serve_scoped};
    Ok(Some(match route {
        Route::Instance(ctx) => serve_scoped(&paths.run, ctx, access_policy(a))?,
        Route::Proxy(up) => serve_proxy(&paths.run, up.clone(), access_policy(a))?,
        Route::Direct(_) => return Ok(None),
    }))
}

/// Run `l` to its end and log how it went (`wall_ms` stamps the log entries).
pub fn launch(
    paths: &AppPaths,
    python: &str,
    l: &Launch,
    route: &Route,
    pause: &dyn Fn(),
    wall_ms: i64,
) -> ScriptOutcome {
    let _ = install_client(&lib_dir(&paths.scripts));
    let socket = match socket_for(paths, route, l.access) {
        Ok(s) => s,
        Err(e) => {
            let o = ScriptOutcome {
                spawn_error: Some(format!("no control socket for the script: {e}")),
                ..ScriptOutcome::default()
            };
            log_outcome(&paths.logs, &l.job, &o, wall_ms);
            return o;
        }
    };
    let path = match (&socket, route) {
        (Some(s), _) => s.path.clone(),
        (None, Route::Direct(p)) => p.clone(),
        (None, _) => PathBuf::new(),
    };
    let outcome = run_script(&command_for(paths, python, l, &path), pause);
    if let Some(s) = &socket {
        crate::script_socket::close_run_socket(s);
    }
    log_outcome(&paths.logs, &l.job, &outcome, wall_ms);
    outcome
}
