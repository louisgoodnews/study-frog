//! Five-minute idle auto-shutdown and memory reclamation supervisor (TASK-0272).
//!
//! Supervises the lifecycle of a managed local SLM sidecar process. Automatically
//! shuts down the child process after 5 minutes (default 300 seconds) of inactivity
//! to reclaim 100% of memory and avoid zombie processes.
//!
//! Adheres strictly to Design Principles:
//! - P1: Zero inherent impl blocks; all operational logic in free functions.
//! - P6: Maximum 250 lines per source file.

use std::process::Child;

use crate::sidecar_types::{SidecarProcessHandle, SidecarStatus};

/// Default inactivity timeout before auto-shutdown: 300 seconds (5 minutes).
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 300;

/// Child process handle abstraction supporting real OS child processes and mock handles.
#[derive(Debug)]
pub enum SidecarChild {
    /// Live operating system process spawned via `std::process::Command`.
    Process(Child),
    /// Lightweight handle or mock process descriptor for tests / external supervision.
    Handle(SidecarProcessHandle),
}

impl From<Child> for SidecarChild {
    fn from(child: Child) -> Self {
        SidecarChild::Process(child)
    }
}

impl From<SidecarProcessHandle> for SidecarChild {
    fn from(handle: SidecarProcessHandle) -> Self {
        SidecarChild::Handle(handle)
    }
}

impl From<crate::sidecar_process::SidecarProcess> for SidecarChild {
    fn from(process: crate::sidecar_process::SidecarProcess) -> Self {
        SidecarChild::Process(process.child)
    }
}

/// Supervisor state machine for a managed local sidecar process.
///
/// Holds current process status, last activity timestamp, idle timeout threshold,
/// and optional child process handle.
#[derive(Debug)]
pub struct SidecarSupervisor {
    /// Current lifecycle status of the sidecar.
    pub status: SidecarStatus,
    /// Unix timestamp in seconds of the most recent user request / interaction.
    pub last_activity_secs: u64,
    /// Idle duration threshold in seconds before triggering automatic shutdown.
    pub idle_timeout_secs: u64,
    /// Handle to the spawned child process, if currently running.
    pub child_process: Option<SidecarChild>,
}

impl Default for SidecarSupervisor {
    fn default() -> Self {
        new_sidecar_supervisor(DEFAULT_IDLE_TIMEOUT_SECS)
    }
}

/// Create a new sidecar supervisor with the specified idle timeout in seconds.
///
/// If `idle_timeout_secs` is 0, defaults to [`DEFAULT_IDLE_TIMEOUT_SECS`] (300 seconds).
pub fn new_sidecar_supervisor(idle_timeout_secs: u64) -> SidecarSupervisor {
    let timeout = if idle_timeout_secs == 0 {
        DEFAULT_IDLE_TIMEOUT_SECS
    } else {
        idle_timeout_secs
    };
    SidecarSupervisor {
        status: SidecarStatus::Stopped,
        last_activity_secs: 0,
        idle_timeout_secs: timeout,
        child_process: None,
    }
}

/// Record user or API activity at the given timestamp in seconds.
///
/// Updates `last_activity_secs` to prevent idle timeout from expiring.
pub fn record_activity(supervisor: &mut SidecarSupervisor, now_secs: u64) {
    supervisor.last_activity_secs = supervisor.last_activity_secs.max(now_secs);
}

/// Check if the supervisor has been idle longer than its configured timeout threshold.
pub fn is_idle_expired(supervisor: &SidecarSupervisor, now_secs: u64) -> bool {
    if supervisor.idle_timeout_secs == 0 {
        return false;
    }
    now_secs.saturating_sub(supervisor.last_activity_secs) >= supervisor.idle_timeout_secs
}

/// Number of seconds remaining until idle shutdown at `now_secs`.
///
/// Returns 0 if already expired or if idle timeout is disabled.
pub fn remaining_idle_secs(supervisor: &SidecarSupervisor, now_secs: u64) -> u64 {
    if supervisor.idle_timeout_secs == 0 {
        return 0;
    }
    let elapsed = now_secs.saturating_sub(supervisor.last_activity_secs);
    supervisor.idle_timeout_secs.saturating_sub(elapsed)
}

/// Check whether the child process is currently alive and executing.
pub fn is_child_alive(child: &mut SidecarChild) -> bool {
    match child {
        SidecarChild::Process(p) => matches!(p.try_wait(), Ok(None)),
        SidecarChild::Handle(_) => true,
    }
}

/// Terminate the child process and reap its exit status to avoid zombie processes.
pub fn stop_child(child: &mut SidecarChild) {
    match child {
        SidecarChild::Process(p) => {
            let _ = p.kill();
            let _ = p.wait();
        }
        SidecarChild::Handle(_) => {}
    }
}

/// Stop the sidecar process and transition status to `Stopped`.
///
/// Drops and reaps the child process handle, reclaiming memory.
pub fn shutdown_sidecar(supervisor: &mut SidecarSupervisor) {
    if let Some(mut child) = supervisor.child_process.take() {
        stop_child(&mut child);
    }
    supervisor.status = SidecarStatus::Stopped;
}

/// Check if the idle timeout has expired, and if so, shut down the child process.
///
/// Returns `true` if an idle shutdown was triggered, `false` otherwise.
pub fn check_idle_shutdown(supervisor: &mut SidecarSupervisor, now_secs: u64) -> bool {
    if supervisor.status == SidecarStatus::Stopped {
        return false;
    }
    if is_idle_expired(supervisor, now_secs) {
        shutdown_sidecar(supervisor);
        true
    } else {
        false
    }
}

/// Periodic supervisor tick routine.
///
/// 1. Inspects the health of the child process if running (transitions to `Failed` if exited).
/// 2. Checks idle timeout and triggers auto-shutdown to reclaim memory if expired.
///
/// Returns `true` if an idle shutdown was executed on this tick.
pub fn supervisor_tick(supervisor: &mut SidecarSupervisor, now_secs: u64) -> bool {
    if supervisor.status == SidecarStatus::Running {
        let dead = supervisor
            .child_process
            .as_mut()
            .is_some_and(|c| !is_child_alive(c));
        if dead {
            supervisor.child_process = None;
            supervisor.status =
                SidecarStatus::Failed("child process exited unexpectedly".to_string());
            return false;
        }
    }
    check_idle_shutdown(supervisor, now_secs)
}

/// Attach a running child process to the supervisor, setting status to `Running`.
pub fn attach_child(supervisor: &mut SidecarSupervisor, child: SidecarChild, now_secs: u64) {
    supervisor.child_process = Some(child);
    supervisor.status = SidecarStatus::Running;
    supervisor.last_activity_secs = now_secs;
}

/// Attach a running process handle to the supervisor, setting status to `Running`.
pub fn attach_process_handle(
    supervisor: &mut SidecarSupervisor,
    handle: SidecarProcessHandle,
    now_secs: u64,
) {
    attach_child(supervisor, SidecarChild::Handle(handle), now_secs);
}

/// Attach a spawned [`crate::sidecar_process::SidecarProcess`] to the supervisor.
pub fn attach_sidecar_process(
    supervisor: &mut SidecarSupervisor,
    process: crate::sidecar_process::SidecarProcess,
    now_secs: u64,
) {
    attach_child(supervisor, SidecarChild::Process(process.child), now_secs);
}

/// Get the current lifecycle status of the sidecar supervisor.
pub fn sidecar_status(supervisor: &SidecarSupervisor) -> &SidecarStatus {
    &supervisor.status
}

/// Check whether the sidecar is currently running and accepting requests.
pub fn is_sidecar_ready(supervisor: &SidecarSupervisor) -> bool {
    supervisor.status == SidecarStatus::Running
}
