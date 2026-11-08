//! A socket of a script's own for one run (ADR 0013): served with the script's access in place of the
//! instance's policy, or, from `studyfrog script run` outside the instance, a proxy to the instance's
//! socket that refuses what the access does not allow. Removed again when the run ends.

use std::fs;
use std::io::{self, BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::control_cmd::{ControlPolicy, allowed};
use crate::control_conn::ConnCtx;
use crate::control_logs::{SharedOut, write_line};
use crate::control_parse::parse_request;
use crate::control_reply::err_line;
use crate::control_socket::serve_client;

/// A socket that lives for one run.
pub struct RunSocket {
    /// Its path (given to the script as `STUDYFROG_SOCKET`).
    pub path: PathBuf,
    /// Set when the run ended, so the accepting thread stops.
    pub stop: Arc<AtomicBool>,
}

/// A fresh socket path in `run`: `script-<pid>-<n>.sock`.
pub fn run_socket_path(run: &Path) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    run.join(format!("script-{}-{n}.sock", std::process::id()))
}

fn bind_private(path: &Path) -> io::Result<UnixListener> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    let _ = fs::remove_file(path);
    let listener = UnixListener::bind(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

fn accept_until_stopped(
    listener: UnixListener,
    stop: Arc<AtomicBool>,
    each: impl Fn(UnixStream) + Send + 'static,
) {
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            if stop.load(Ordering::Relaxed) {
                return;
            }
            each(stream);
        }
    });
}

/// Serve a socket in `run` whose requests run under `policy` (in this instance).
pub fn serve_scoped(run: &Path, ctx: &ConnCtx, policy: ControlPolicy) -> io::Result<RunSocket> {
    let path = run_socket_path(run);
    let listener = bind_private(&path)?;
    let stop = Arc::new(AtomicBool::new(false));
    let ctx = ConnCtx {
        scope: Some(policy),
        ..ctx.clone()
    };
    accept_until_stopped(listener, stop.clone(), move |stream| {
        let ctx = ctx.clone();
        std::thread::spawn(move || serve_client(&ctx, stream));
    });
    Ok(RunSocket { path, stop })
}

/// Copy one client to `upstream`, answering requests `policy` refuses itself.
fn proxy_client(client: UnixStream, upstream: &Path, policy: ControlPolicy) -> io::Result<()> {
    let up = UnixStream::connect(upstream)?;
    let out: SharedOut = Arc::new(Mutex::new(Box::new(client.try_clone()?)));
    let (back, up_reader) = (out.clone(), up.try_clone()?);
    std::thread::spawn(move || {
        for line in BufReader::new(up_reader).lines().map_while(Result::ok) {
            if write_line(&back, &line).is_err() {
                return;
            }
        }
    });
    let up_out: SharedOut = Arc::new(Mutex::new(Box::new(up)));
    for line in BufReader::new(client).lines().map_while(Result::ok) {
        let refused = match parse_request(&line) {
            Ok(req) => allowed(&policy, &req.cmd)
                .err()
                .map(|why| err_line(&req.id, &why)),
            Err(_) => None,
        };
        match refused {
            Some(answer) => write_line(&out, &answer)?,
            None => write_line(&up_out, &line)?,
        }
    }
    Ok(())
}

/// Serve a socket at a fresh path in `run` that forwards to `upstream` what `policy` allows.
pub fn serve_proxy(run: &Path, upstream: PathBuf, policy: ControlPolicy) -> io::Result<RunSocket> {
    let path = run_socket_path(run);
    let listener = bind_private(&path)?;
    let stop = Arc::new(AtomicBool::new(false));
    accept_until_stopped(listener, stop.clone(), move |stream| {
        let up = upstream.clone();
        std::thread::spawn(move || proxy_client(stream, &up, policy));
    });
    Ok(RunSocket { path, stop })
}

/// End the socket: stop accepting and remove the file.
pub fn close_run_socket(s: &RunSocket) {
    s.stop.store(true, Ordering::Relaxed);
    let _ = UnixStream::connect(&s.path);
    let _ = fs::remove_file(&s.path);
}
