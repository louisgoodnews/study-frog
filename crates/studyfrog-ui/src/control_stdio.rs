//! `studyfrog --control-stdio [--vault PATH] [--control-read-only] [--control-allow-destructive]
//! [--no-control-socket]`: a headless instance driven by control requests, one per line on standard
//! input, answered on standard output; it also serves `run/control.sock` for `studyfrog ctl`.
//! Diagnostics go to standard error only.

use std::io::{self, BufReader};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::control_cmd::{ControlPolicy, policy_default};
use crate::control_conn::{ConnCtx, Job, serve_connection};
use crate::control_engine::{Clock, Engine, engine_loop, run_phase, settle};
use crate::control_logs::SharedOut;
use crate::effect::Effect;
use crate::exec::worker_with_host;
use crate::host::{Host, extensions_of};
use crate::model::Model;
use crate::msg::Msg;
use crate::runtime::paths;

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ControlArgs {
    /// The vault to open; else the one opened last.
    pub vault: Option<String>,
    /// What requests may change.
    pub policy: ControlPolicy,
    /// Do not serve `run/control.sock`.
    pub no_socket: bool,
}

/// Read the flags after the program name; `--control-stdio` itself is skipped.
pub fn parse_control_args(args: &[String]) -> Result<ControlArgs, String> {
    let mut out = ControlArgs {
        policy: policy_default(),
        ..ControlArgs::default()
    };
    let mut rest = args.iter().filter(|a| *a != "--control-stdio");
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--vault" => out.vault = Some(rest.next().ok_or("--vault needs a folder")?.clone()),
            "--control-read-only" => out.policy.allow_write = false,
            "--control-allow-destructive" => out.policy.allow_destructive = true,
            "--no-control-socket" => out.no_socket = true,
            other => return Err(format!("unknown option {other}")),
        }
    }
    Ok(out)
}

fn wall_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// The real clock: seconds since `start`, and the wall clock.
pub fn system_clock() -> Clock {
    let start = Instant::now();
    Box::new(move || (start.elapsed().as_secs_f64(), wall_ms()))
}

/// A headless engine with the host's runtimes (no dialogs), its settings loaded and `vault` opened.
pub fn engine_new(
    host: Host,
    policy: ControlPolicy,
    clock: Clock,
    vault: Option<String>,
) -> Engine {
    let p = paths();
    let host = Host {
        folder_picker: None,
        file_picker: None,
        ..host
    };
    let mut model = Model::default();
    model.state.extensions = extensions_of(&host.registry);
    let mut worker = worker_with_host(
        p.as_ref().map(|p| p.config.clone()),
        p.as_ref().map(|p| p.models.clone()),
        host,
    );
    worker.state_dir = p.as_ref().map(|p| p.logs.clone());
    let logs = worker.state_dir.clone();
    let mut e = Engine {
        model,
        worker,
        policy,
        logs,
        clock,
    };
    settle(&mut e, vec![Effect::LoadAppSettings]);
    e.model.state.app.tutorial_offered = true;
    let path = vault.or_else(|| e.model.state.app.vault.clone());
    if let Some(path) = path {
        run_phase(&mut e, vec![Msg::OpenRecent(path)]);
    }
    e
}

#[cfg(unix)]
fn serve_socket(ctx: &ConnCtx, vault: Option<&str>) -> Option<crate::control_socket::ControlGuard> {
    use crate::control_socket::{bind, publish, serve};
    let run: PathBuf = paths()?.run;
    let bound = bind(&run).and_then(|l| Ok((l, publish(&run, "headless", vault, wall_ms())?)));
    match bound {
        Ok((listener, guard)) => {
            serve(listener, ctx.clone());
            Some(guard)
        }
        Err(e) => {
            eprintln!("studyfrog --control-stdio: no control socket: {e}");
            None
        }
    }
}

#[cfg(not(unix))]
fn serve_socket(_: &ConnCtx, _: Option<&str>) -> Option<()> {
    None
}

/// Run the headless instance until standard input ends; the exit code.
pub fn run(args: &[String], host: Host) -> u8 {
    let a = match parse_control_args(args.get(1..).unwrap_or_default()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("studyfrog --control-stdio: {e}");
            return 2;
        }
    };
    let mut engine = engine_new(host, a.policy, system_clock(), a.vault.clone());
    if let Some(n) = engine.model.state.notice.as_ref() {
        eprintln!("studyfrog --control-stdio: {}", n.text);
    }
    let (inbox, jobs) = channel();
    let ctx = ConnCtx {
        inbox,
        wake: Arc::new(|| {}),
        logs: engine.logs.clone(),
        wait: crate::cli::pause,
    };
    let vault = engine.model.state.vault.as_ref().map(|v| v.root.clone());
    let _guard = if a.no_socket {
        None
    } else {
        serve_socket(&ctx, vault.as_deref())
    };
    let stdin_ctx = ctx.clone();
    std::thread::spawn(move || {
        let out: SharedOut = Arc::new(Mutex::new(Box::new(io::stdout())));
        serve_connection(&stdin_ctx, BufReader::new(io::stdin()), &out);
        let _ = stdin_ctx.inbox.send(Job::Quit);
    });
    drop(ctx);
    engine_loop(&mut engine, &jobs);
    0
}
