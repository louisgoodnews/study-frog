//! Sidecar child process spawning, port binding, and stdin watchdog (TASK-0271).
//!
//! Enforces Invariants:
//! - P1: Zero inherent impl blocks; pure free functions only.
//! - P6: Maximum 250 lines.

use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};

use crate::sidecar_types::{
    PortError, PortRange, SidecarConfig, SidecarError, validate_sidecar_config,
};

/// Live spawned child process representation with loopback port and OS process handle.
#[derive(Debug)]
pub struct SidecarProcess {
    /// Process ID of the spawned child.
    pub pid: u32,
    /// Loopback port assigned to the process.
    pub port: u16,
    /// Underlying OS child process handle.
    pub child: Child,
}

/// Allocates an ephemeral loopback port by temporarily binding to port 0.
pub fn allocate_ephemeral_port() -> Result<u16, PortError> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| PortError::BindFailed(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| PortError::BindFailed(e.to_string()))?
        .port();
    drop(listener);
    Ok(port)
}

/// Checks whether a specific host and port combination is available for TCP binding.
pub fn is_port_available(host: &str, port: u16) -> bool {
    std::net::TcpListener::bind((host, port)).is_ok()
}

/// Finds the first available loopback port within the specified [`PortRange`].
pub fn find_available_loopback_port(range: &PortRange) -> Result<u16, PortError> {
    if range.min_port == 0 {
        return allocate_ephemeral_port();
    }
    for port in range.min_port..=range.max_port {
        if is_port_available("127.0.0.1", port) {
            return Ok(port);
        }
    }
    Err(PortError::NoPortAvailable(range.min_port, range.max_port))
}

/// Builds a [`Command`] configured with arguments, env vars, and piped stdin watchdog.
pub fn build_sidecar_command(config: &SidecarConfig, port: u16) -> Command {
    let mut cmd = Command::new(&config.binary_path);
    if config.pass_server_args {
        cmd.arg("--host").arg(&config.host);
        cmd.arg("--port").arg(port.to_string());
        if !config.model_path.trim().is_empty() {
            cmd.arg("-m").arg(&config.model_path);
        }
        if config.context_size > 0 {
            cmd.arg("-c").arg(config.context_size.to_string());
        }
        if let Some(threads) = config.threads {
            cmd.arg("-t").arg(threads.to_string());
        }
    }
    for arg in &config.extra_args {
        cmd.arg(arg);
    }
    cmd.stdin(Stdio::piped());
    if let Some(ref api_key) = config.api_key {
        cmd.env("LLAMA_API_KEY", api_key);
    }
    for (k, v) in &config.env_vars {
        cmd.env(k, v);
    }
    cmd
}

/// Spawns the background sidecar process with ephemeral port and stdin watchdog.
pub fn spawn_sidecar_process(config: &SidecarConfig) -> Result<SidecarProcess, SidecarError> {
    validate_sidecar_config(config)?;
    let port = if config.port > 0 {
        config.port
    } else {
        find_available_loopback_port(&config.port_range)
            .map_err(|e| SidecarError::PortUnavailable(e.to_string()))?
    };
    let mut cmd = build_sidecar_command(config, port);
    cmd.stdout(Stdio::null());
    let child = cmd
        .spawn()
        .map_err(|e| SidecarError::ProcessSpawnFailed(e.to_string()))?;
    let pid = child.id();
    Ok(SidecarProcess { pid, port, child })
}

/// Closes the write end of the parent-to-child stdin pipe, signaling EOF to the watchdog.
pub fn close_stdin(process: &mut SidecarProcess) -> bool {
    process.child.stdin.take().is_some()
}

/// Checks whether a child process is currently alive and executing.
pub fn is_sidecar_process_alive(process: &mut SidecarProcess) -> bool {
    matches!(process.child.try_wait(), Ok(None))
}

/// Checks whether an arbitrary OS process PID is currently alive without zombie status.
pub fn is_process_alive(pid: u32) -> bool {
    let proc_path = Path::new("/proc").join(pid.to_string());
    if proc_path.exists() {
        if let Ok(status) = std::fs::read_to_string(proc_path.join("status")) {
            for line in status.lines() {
                if let Some(state) = line.strip_prefix("State:") {
                    if state.trim().starts_with('Z') {
                        return false;
                    }
                    break;
                }
            }
        }
        return true;
    }
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Checks whether the running process's executable path contains the expected name.
pub fn process_executable_matches(pid: u32, expected_name: &str) -> bool {
    let exe_path = format!("/proc/{pid}/exe");
    std::fs::read_link(exe_path)
        .ok()
        .and_then(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().contains(expected_name))
        })
        .unwrap_or(false)
}

/// Waits for a sidecar process to exit, returning its exit status.
pub fn wait_for_process_exit(process: &mut SidecarProcess) -> Result<ExitStatus, SidecarError> {
    process
        .child
        .wait()
        .map_err(|e| SidecarError::Io(e.to_string()))
}

/// Checks if the process has already exited without blocking.
pub fn try_wait_process(process: &mut SidecarProcess) -> Result<Option<ExitStatus>, SidecarError> {
    process
        .child
        .try_wait()
        .map_err(|e| SidecarError::Io(e.to_string()))
}

/// Terminates a sidecar child process gracefully via stdin EOF and signals.
pub fn kill_sidecar_process(process: &mut SidecarProcess) -> Result<(), SidecarError> {
    close_stdin(process);
    if let Ok(Some(_)) = process.child.try_wait() {
        return Ok(());
    }
    let _ = Command::new("kill")
        .args(["-TERM", &process.pid.to_string()])
        .output();
    if let Ok(Some(_)) = process.child.try_wait() {
        return Ok(());
    }
    let _ = process.child.kill();
    let _ = process.child.wait();
    Ok(())
}

/// Terminates any process by PID after attempting SIGTERM then SIGKILL.
pub fn kill_process_by_pid(pid: u32) -> Result<(), SidecarError> {
    if !is_process_alive(pid) {
        return Ok(());
    }
    let _ = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .output();
    if is_process_alive(pid) {
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
    }
    Ok(())
}
