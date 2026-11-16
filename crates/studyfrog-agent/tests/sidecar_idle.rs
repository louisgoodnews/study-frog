//! Integration tests for the 5-minute idle auto-shutdown supervisor (TASK-0272).
//!
//! Follows Invariant P1 (zero inherent impls) and P6 (<= 250 lines).

use std::process::Command;
use studyfrog_agent::sidecar_supervisor::{
    DEFAULT_IDLE_TIMEOUT_SECS, SidecarChild, attach_child, attach_process_handle,
    check_idle_shutdown, is_idle_expired, is_sidecar_ready, new_sidecar_supervisor,
    record_activity, remaining_idle_secs, shutdown_sidecar, sidecar_status, supervisor_tick,
};
use studyfrog_agent::sidecar_types::{SidecarProcessHandle, SidecarStatus};

#[test]
fn default_supervisor_initializes_with_five_minute_timeout() {
    let supervisor = new_sidecar_supervisor(DEFAULT_IDLE_TIMEOUT_SECS);
    assert_eq!(supervisor.idle_timeout_secs, 300);
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Stopped);
    assert_eq!(supervisor.last_activity_secs, 0);
    assert!(supervisor.child_process.is_none());
    assert!(!is_sidecar_ready(&supervisor));

    // Passing 0 also defaults to 300 seconds
    let def = new_sidecar_supervisor(0);
    assert_eq!(def.idle_timeout_secs, 300);
}

#[test]
fn idle_expiration_and_remaining_seconds_math() {
    let mut supervisor = new_sidecar_supervisor(300);
    record_activity(&mut supervisor, 1_000);

    assert_eq!(remaining_idle_secs(&supervisor, 1_000), 300);
    assert_eq!(remaining_idle_secs(&supervisor, 1_100), 200);
    assert_eq!(remaining_idle_secs(&supervisor, 1_299), 1);
    assert_eq!(remaining_idle_secs(&supervisor, 1_300), 0);
    assert_eq!(remaining_idle_secs(&supervisor, 1_500), 0);

    assert!(!is_idle_expired(&supervisor, 1_000));
    assert!(!is_idle_expired(&supervisor, 1_299));
    assert!(is_idle_expired(&supervisor, 1_300));
    assert!(is_idle_expired(&supervisor, 1_400));

    // Underflow protection when clock jitter occurs
    assert_eq!(remaining_idle_secs(&supervisor, 500), 300);
    assert!(!is_idle_expired(&supervisor, 500));
}

#[test]
fn activity_recording_resets_idle_timer() {
    let mut supervisor = new_sidecar_supervisor(300);
    record_activity(&mut supervisor, 1_000);

    // After 250s, new activity occurs
    record_activity(&mut supervisor, 1_250);
    assert!(!is_idle_expired(&supervisor, 1_300));
    assert_eq!(remaining_idle_secs(&supervisor, 1_300), 250);

    // Timeout now happens at 1250 + 300 = 1550
    assert!(!is_idle_expired(&supervisor, 1_549));
    assert!(is_idle_expired(&supervisor, 1_550));
}

#[test]
fn auto_shutdown_stops_mock_child_and_transitions_to_stopped() {
    let mut supervisor = new_sidecar_supervisor(300);
    let handle = SidecarProcessHandle {
        pid: 9999,
        port: 18080,
    };
    attach_process_handle(&mut supervisor, handle, 1_000);

    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Running);
    assert!(is_sidecar_ready(&supervisor));
    assert!(supervisor.child_process.is_some());

    // Tick before timeout does not trigger shutdown
    assert!(!supervisor_tick(&mut supervisor, 1_200));
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Running);
    assert!(supervisor.child_process.is_some());

    // Tick at 5 minutes idle triggers auto-shutdown
    assert!(supervisor_tick(&mut supervisor, 1_300));
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Stopped);
    assert!(!is_sidecar_ready(&supervisor));
    assert!(supervisor.child_process.is_none());

    // Subsequent tick on stopped supervisor does not trigger again
    assert!(!supervisor_tick(&mut supervisor, 1_301));
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Stopped);
}

#[test]
fn manual_shutdown_reclaims_process() {
    let mut supervisor = new_sidecar_supervisor(300);
    attach_process_handle(
        &mut supervisor,
        SidecarProcessHandle {
            pid: 1234,
            port: 18080,
        },
        500,
    );
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Running);

    shutdown_sidecar(&mut supervisor);
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Stopped);
    assert!(supervisor.child_process.is_none());
}

#[test]
fn real_os_child_process_killed_and_reaped_on_idle_shutdown() {
    // Spawn a long-running real OS child process (sleep 60)
    let child = Command::new("sleep")
        .arg("60")
        .spawn()
        .expect("failed to spawn test sleep process");
    let pid = child.id();

    let mut supervisor = new_sidecar_supervisor(300);
    attach_child(&mut supervisor, SidecarChild::Process(child), 1_000);

    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Running);
    assert!(supervisor.child_process.is_some());

    // Before 5 minutes, process stays running
    let shut = check_idle_shutdown(&mut supervisor, 1_250);
    assert!(!shut);
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Running);

    // Exactly at 5 minutes (300s), check_idle_shutdown triggers
    let shut = check_idle_shutdown(&mut supervisor, 1_300);
    assert!(shut);
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Stopped);
    assert!(supervisor.child_process.is_none());

    // Verify OS process has been killed and reaped
    // In Linux, sending signal 0 via kill check confirms process is gone
    let proc_status = Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .status();
    assert!(
        proc_status.map_or(true, |s| !s.success()),
        "Child process {pid} was not killed/reaped on idle auto-shutdown"
    );
}

#[test]
#[allow(clippy::disallowed_methods)]
fn supervisor_tick_detects_unexpected_child_exit() {
    // Spawn a process that exits immediately
    let child = Command::new("true").spawn().expect("failed to spawn true");

    let mut supervisor = new_sidecar_supervisor(300);
    attach_child(&mut supervisor, SidecarChild::Process(child), 1_000);

    // Give process a moment to exit
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Tick before idle timeout detects child exit and transitions to Failed
    let shut = supervisor_tick(&mut supervisor, 1_050);
    assert!(!shut);
    assert!(matches!(
        sidecar_status(&supervisor),
        SidecarStatus::Failed(_)
    ));
    assert!(supervisor.child_process.is_none());
}

#[test]
fn custom_idle_timeout_shuts_down_early() {
    // Custom 60-second idle timeout
    let mut supervisor = new_sidecar_supervisor(60);
    attach_process_handle(
        &mut supervisor,
        SidecarProcessHandle {
            pid: 5678,
            port: 18080,
        },
        100,
    );

    assert!(!check_idle_shutdown(&mut supervisor, 159));
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Running);

    assert!(check_idle_shutdown(&mut supervisor, 160));
    assert_eq!(*sidecar_status(&supervisor), SidecarStatus::Stopped);
    assert!(supervisor.child_process.is_none());
}
