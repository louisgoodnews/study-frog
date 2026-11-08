//! The control socket of a running instance: `<app home>/run/control.sock` (only the user may open
//! it) with `run/control.json` naming the process that serves it. A socket left by an instance that
//! is gone is replaced; one that answers belongs to another instance and is left alone.

use std::fs;
use std::io::{self, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::control_conn::{ConnCtx, serve_connection};
use crate::control_logs::SharedOut;

/// The socket's file name inside `run/`.
pub const SOCKET_FILE: &str = "control.sock";
/// The file naming the serving process, inside `run/`.
pub const META_FILE: &str = "control.json";

/// The socket and its metadata file; removed again when this is dropped.
pub struct ControlGuard {
    /// The socket.
    pub socket: PathBuf,
    /// The metadata.
    pub meta: PathBuf,
}

impl Drop for ControlGuard {
    fn drop(&mut self) {
        release(self);
    }
}

/// Remove the socket and the metadata, if they are still ours.
pub fn release(g: &ControlGuard) {
    let ours = fs::read_to_string(&g.meta)
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.get("pid").and_then(serde_json::Value::as_u64))
        == Some(u64::from(std::process::id()));
    if ours {
        let _ = fs::remove_file(&g.socket);
        let _ = fs::remove_file(&g.meta);
    }
}

/// The socket path inside a `run/` directory.
pub fn socket_path(run: &Path) -> PathBuf {
    run.join(SOCKET_FILE)
}

/// Bind the socket in `run`, creating the directory (only the user may enter it). Fails when
/// another instance answers on it.
pub fn bind(run: &Path) -> io::Result<UnixListener> {
    fs::create_dir_all(run)?;
    fs::set_permissions(run, fs::Permissions::from_mode(0o700))?;
    let path = socket_path(run);
    if path.exists() {
        if UnixStream::connect(&path).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                format!("another StudyFrog instance is serving {}", path.display()),
            ));
        }
        fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

/// Write `run/control.json` and return the guard that removes both files again.
pub fn publish(
    run: &Path,
    mode: &str,
    vault: Option<&str>,
    now_ms: i64,
) -> io::Result<ControlGuard> {
    let socket = socket_path(run);
    let meta = run.join(META_FILE);
    let text = serde_json::json!({
        "pid": std::process::id(), "socket": socket, "mode": mode, "vault": vault,
        "started_ms": now_ms, "version": env!("CARGO_PKG_VERSION"), "protocol": 1,
    });
    fs::write(&meta, format!("{text}\n"))?;
    Ok(ControlGuard { socket, meta })
}

fn client(ctx: &ConnCtx, stream: UnixStream) {
    let Ok(reader) = stream.try_clone() else {
        return;
    };
    let out: SharedOut = Arc::new(Mutex::new(Box::new(stream)));
    serve_connection(ctx, BufReader::new(reader), &out);
}

/// Accept clients on a thread of their own, each served on its own thread.
pub fn serve(listener: UnixListener, ctx: ConnCtx) {
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let ctx = ctx.clone();
            std::thread::spawn(move || client(&ctx, stream));
        }
    });
}
