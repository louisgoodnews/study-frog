//! `studyfrog script ...` (ADR 0013): `run FILE|JOB [--access read|write|destructive] [--timeout S]
//! [-- ARGS]`, `list`, `daemon [--vault PATH]` and `install-client [DIR]`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use studyfrog_config::paths::AppPaths;
use studyfrog_service::scripting::config::{Access, DEFAULT_TIMEOUT_SECS};
use studyfrog_service::scripting::due::find_job;
use studyfrog_service::scripting::exec::{ScriptOutcome, succeeded};

use crate::host::Host;
use crate::runtime::paths;
use crate::script_launch::{Launch, Route, launch};
use crate::script_runner::{child_pause, launch_of, load_config, wall_ms};

/// What `script run` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunArgs {
    /// A job's name or a script file.
    pub what: String,
    /// `--access`.
    pub access: Option<Access>,
    /// `--timeout`.
    pub timeout: Option<u64>,
    /// The words after `--`.
    pub args: Vec<String>,
}

/// Read the words after `script run`.
pub fn parse_run_args(words: &[String]) -> Result<RunArgs, String> {
    let (mine, args) = match words.iter().position(|w| w == "--") {
        Some(i) => (&words[..i], words[i + 1..].to_vec()),
        None => (words, Vec::new()),
    };
    let mut out = RunArgs {
        what: String::new(),
        access: None,
        timeout: None,
        args,
    };
    let mut rest = mine.iter();
    while let Some(w) = rest.next() {
        match w.as_str() {
            "--access" => {
                out.access = Some(match rest.next().map(String::as_str) {
                    Some("read") => Access::Read,
                    Some("write") => Access::Write,
                    Some("destructive") => Access::Destructive,
                    other => {
                        return Err(format!(
                            "--access {} (read, write, destructive)",
                            other.unwrap_or("")
                        ));
                    }
                });
            }
            "--timeout" => {
                let t = rest.next().ok_or("--timeout needs seconds")?;
                out.timeout = Some(
                    t.parse()
                        .map_err(|_| format!("--timeout {t}: not a number"))?,
                );
            }
            other if out.what.is_empty() => other.clone_into(&mut out.what),
            other => out.args.push(other.to_owned()),
        }
    }
    if out.what.is_empty() {
        return Err("give a script file or a job of scripts.toml".into());
    }
    Ok(out)
}

/// The run `a` asks for: a configured job (its access and timeout unless overridden) or a file
/// (relative to the current folder when it is there, else to the scripts folder; write access).
pub fn launch_for(p: &AppPaths, a: &RunArgs) -> Launch {
    let cfg = load_config(&p.scripts);
    let mut l = match find_job(&cfg, &a.what) {
        Some(job) => launch_of(job, None),
        None => {
            let here = Path::new(&a.what);
            let script = if here.is_file() {
                std::fs::canonicalize(here)
                    .map_or_else(|_| a.what.clone(), |x| x.display().to_string())
            } else {
                a.what.clone()
            };
            Launch {
                job: here
                    .file_name()
                    .map_or(a.what.clone(), |n| n.to_string_lossy().into_owned()),
                script,
                args: Vec::new(),
                access: Access::Write,
                timeout_secs: DEFAULT_TIMEOUT_SECS,
                event: None,
            }
        }
    };
    l.access = a.access.unwrap_or(l.access);
    l.timeout_secs = a.timeout.unwrap_or(l.timeout_secs);
    if !a.args.is_empty() {
        l.args.clone_from(&a.args);
    }
    l
}

fn answers(socket: &Path) -> bool {
    std::os::unix::net::UnixStream::connect(socket).is_ok()
}

/// A headless instance for the run, serving `run/control.sock` with the access as its policy.
fn start_headless(p: &AppPaths, access: Access) -> Result<Child, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = Command::new(exe);
    cmd.args(["--control-stdio", "--no-scripts"]);
    match access {
        Access::Read => {
            cmd.arg("--control-read-only");
        }
        Access::Write => {}
        Access::Destructive => {
            cmd.arg("--control-allow-destructive");
        }
    }
    let child = cmd
        .env("STUDYFROG_HOME", &p.root)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| e.to_string())?;
    let meta = p.run.join(crate::control_socket::META_FILE);
    for _ in 0..400 {
        if meta.exists() {
            return Ok(child);
        }
        child_pause();
    }
    Err("the headless instance did not start its control socket".into())
}

fn exit_code(o: &ScriptOutcome) -> u8 {
    if succeeded(o) {
        return 0;
    }
    o.code
        .and_then(|c| u8::try_from(c).ok())
        .filter(|c| *c != 0)
        .unwrap_or(1)
}

fn run(p: &AppPaths, words: &[String]) -> Result<u8, String> {
    let a = parse_run_args(words)?;
    let l = launch_for(p, &a);
    let cfg = load_config(&p.scripts);
    let socket = p.run.join(crate::control_socket::SOCKET_FILE);
    let (route, mut child) = if answers(&socket) {
        (Route::Proxy(socket), None)
    } else {
        (Route::Direct(socket), Some(start_headless(p, l.access)?))
    };
    let o = launch(p, &cfg.python, &l, &route, &child_pause, wall_ms());
    if let Some(c) = child.as_mut() {
        drop(c.stdin.take());
        let _ = c.wait();
    }
    print!("{}", o.stdout);
    eprint!("{}", o.stderr);
    if !succeeded(&o) {
        eprintln!(
            "studyfrog script: {}",
            studyfrog_service::scripting::log::outcome_line(&l.job, &o)
        );
    }
    Ok(exit_code(&o))
}

/// Run `studyfrog script <words>`; the exit code.
pub fn run_cli(words: &[String], host: fn() -> Host) -> u8 {
    let Some(p) = paths() else {
        eprintln!("studyfrog script: no application home is known");
        return 2;
    };
    let rest = words.get(1..).unwrap_or_default();
    let result = match words.first().map(String::as_str) {
        Some("run") => run(&p, rest),
        Some("list") => {
            let (text, ok) = crate::script_runner::list_text(&p);
            print!("{text}");
            Ok(u8::from(!ok))
        }
        Some("daemon") => {
            let mut args = vec!["studyfrog".to_owned(), "--control-stdio".to_owned(), "--daemon".to_owned()];
            args.extend(rest.iter().cloned());
            Ok(crate::control_stdio::run(&args, host()))
        }
        Some("install-client") => {
            let dir = rest.first().map_or_else(|| crate::script_client::lib_dir(&p.scripts), PathBuf::from);
            crate::script_client::install_client(&dir)
                .map(|n| {
                    let _ = writeln!(std::io::stdout(), "{} ({n} files written)", dir.join("studyfrog").display());
                    0
                })
                .map_err(|e| e.to_string())
        }
        _ => Err("studyfrog script run FILE|JOB [--access read|write|destructive] [--timeout S] [-- ARGS] | list | daemon [--vault PATH] | install-client [DIR]".into()),
    };
    result.unwrap_or_else(|e| {
        eprintln!("studyfrog script: {e}");
        2
    })
}
