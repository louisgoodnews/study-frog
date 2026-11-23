//! Running a read-only system command with a time limit. The command runs on its own thread; when
//! the limit passes the answer is "failed" and the thread is left to finish by itself.

use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Why a command gave no output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunError {
    /// The program is not installed (or not on the search path).
    NotFound,
    /// It failed, exited with an error, or did not finish in time.
    Failed,
}

/// The standard output of `program args` when it exits successfully within `secs` seconds.
pub fn run_timeout(program: &str, args: &[&str], secs: u64) -> Result<String, RunError> {
    let (tx, rx) = mpsc::channel();
    let program = program.to_owned();
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    thread::spawn(move || {
        let out = Command::new(&program).args(&args).output();
        let _ = tx.send(out);
    });
    match rx.recv_timeout(Duration::from_secs(secs)) {
        Ok(Ok(out)) if out.status.success() => {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        }
        Ok(Err(e)) if e.kind() == std::io::ErrorKind::NotFound => Err(RunError::NotFound),
        _ => Err(RunError::Failed),
    }
}
