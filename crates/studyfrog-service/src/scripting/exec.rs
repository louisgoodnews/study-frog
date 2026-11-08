//! Run one script as a child process: no standard input, standard output and error captured (each
//! kept up to [`MAX_OUTPUT`] bytes), killed when it runs over its timeout. The pause between looks at
//! the child is passed in, so tests and the runner choose how to wait.

use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Bytes of each output stream that are kept; the rest is read and dropped.
pub const MAX_OUTPUT: usize = 64 * 1024;
/// How long the output is still read after the script ended.
const OUTPUT_GRACE: Duration = Duration::from_millis(500);

/// What to start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    /// The program (`python3`, or the script itself).
    pub program: String,
    /// Its arguments (the script path first when an interpreter runs it).
    pub args: Vec<String>,
    /// The working directory.
    pub cwd: PathBuf,
    /// Variables added to the environment.
    pub env: Vec<(String, String)>,
    /// How long it may run.
    pub timeout: Duration,
}

/// How a run ended.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScriptOutcome {
    /// The exit code; `None` when it was killed or could not start.
    pub code: Option<i32>,
    /// It ran over its timeout and was killed.
    pub timed_out: bool,
    /// Why it could not be started.
    pub spawn_error: Option<String>,
    /// Standard output (lossy UTF-8, cut at [`MAX_OUTPUT`]).
    pub stdout: String,
    /// Standard error (lossy UTF-8, cut at [`MAX_OUTPUT`]).
    pub stderr: String,
    /// How long it ran, in milliseconds.
    pub millis: u64,
}

/// Whether the run succeeded (started, finished in time, exit code 0).
pub fn succeeded(o: &ScriptOutcome) -> bool {
    o.code == Some(0) && !o.timed_out && o.spawn_error.is_none()
}

type Shared = Arc<Mutex<Vec<u8>>>;

/// Read `stream` into a shared buffer on a thread of its own; `done` is told when it ends.
fn drain(stream: Option<impl Read + Send + 'static>, done: Sender<()>) -> Shared {
    let kept: Shared = Arc::new(Mutex::new(Vec::new()));
    let buf = kept.clone();
    std::thread::spawn(move || {
        let mut chunk = [0u8; 8192];
        let Some(mut s) = stream else {
            let _ = done.send(());
            return;
        };
        while let Ok(n) = s.read(&mut chunk) {
            if n == 0 {
                break;
            }
            if let Ok(mut b) = buf.lock() {
                let room = MAX_OUTPUT.saturating_sub(b.len()).min(n);
                b.extend_from_slice(&chunk[..room]);
            }
        }
        let _ = done.send(());
    });
    kept
}

fn text(buf: &Shared) -> String {
    buf.lock()
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default()
}

/// Kill the child and, on Unix, the process group it leads (programs it started).
fn kill_all(child: &mut Child) {
    #[cfg(unix)]
    {
        let group = format!("-{}", child.id());
        let _ = Command::new("kill").args(["-KILL", "--", &group]).status();
    }
    let _ = child.kill();
}

/// Wait for the child until `timeout`, looking every `pause`; true when it had to be killed.
fn wait_or_kill(child: &mut Child, timeout: Duration, started: Instant, pause: &dyn Fn()) -> bool {
    loop {
        if let Ok(Some(_)) | Err(_) = child.try_wait() {
            return false;
        }
        if started.elapsed() >= timeout {
            kill_all(child);
            return true;
        }
        pause();
    }
}

/// Start `run`, wait for it (killing it at its timeout) and collect what it printed.
pub fn run_script(run: &ScriptRun, pause: &dyn Fn()) -> ScriptOutcome {
    let started = Instant::now();
    let mut cmd = Command::new(&run.program);
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    let spawned = cmd
        .args(&run.args)
        .current_dir(&run.cwd)
        .envs(run.env.iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match spawned {
        Ok(c) => c,
        Err(e) => {
            return ScriptOutcome {
                spawn_error: Some(format!("{}: {e}", run.program)),
                ..ScriptOutcome::default()
            };
        }
    };
    let (done, ended) = channel();
    let out = drain(child.stdout.take(), done.clone());
    let err = drain(child.stderr.take(), done);
    let timed_out = wait_or_kill(&mut child, run.timeout, started, pause);
    let status = child.wait().ok();
    for _ in 0..2 {
        // Programs the script left running may hold its output open: wait a moment, not forever.
        let _ = ended.recv_timeout(OUTPUT_GRACE);
    }
    ScriptOutcome {
        code: status.and_then(|s| s.code()).filter(|_| !timed_out),
        timed_out,
        spawn_error: None,
        stdout: text(&out),
        stderr: text(&err),
        millis: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
    }
}
