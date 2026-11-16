//! Integration tests for sidecar process lifecycle, ports, and stdin watchdog (TASK-0271).

use studyfrog_agent::assistant_backend::{
    AssistantBackend, BackendConfig, BackendError, BackendStatus, new_backend_config,
};
use studyfrog_agent::sidecar_process::{
    allocate_ephemeral_port, build_sidecar_command, close_stdin, find_available_loopback_port,
    is_port_available, is_process_alive, is_sidecar_process_alive, kill_sidecar_process,
    spawn_sidecar_process, wait_for_process_exit,
};
use studyfrog_agent::sidecar_types::{
    PortError, SidecarError, SidecarStatus, default_port_range, default_sidecar_config,
    is_loopback_host, is_sidecar_running, new_port_range, new_sidecar_config, port_range_contains,
    sidecar_config_with_api_key, sidecar_config_with_port, sidecar_status_to_backend_status,
    validate_sidecar_config,
};

#[test]
fn test_loopback_host_validation() {
    assert!(is_loopback_host("127.0.0.1"));
    assert!(is_loopback_host("localhost"));
    assert!(is_loopback_host("::1"));
    assert!(!is_loopback_host("0.0.0.0"));
    assert!(!is_loopback_host("192.168.1.100"));
}

#[test]
fn test_port_range_and_allocation() {
    let r = new_port_range(18000, 18010).unwrap();
    assert!(port_range_contains(&r, 18000));
    assert!(port_range_contains(&r, 18005));
    assert!(!port_range_contains(&r, 17999));
    assert!(!port_range_contains(&r, 18011));

    assert!(matches!(
        new_port_range(0, 100),
        Err(PortError::InvalidRange(0, 100))
    ));
    assert!(matches!(
        new_port_range(100, 50),
        Err(PortError::InvalidRange(100, 50))
    ));

    let port = allocate_ephemeral_port().unwrap();
    assert!(port > 0);

    let def_range = default_port_range();
    let found = find_available_loopback_port(&def_range).unwrap();
    assert!(port_range_contains(&def_range, found));
    assert!(is_port_available("127.0.0.1", found));
}

#[test]
fn test_sidecar_config_and_validation() {
    let mut cfg = default_sidecar_config();
    assert_eq!(cfg.binary_path, "llama-server");
    assert_eq!(cfg.host, "127.0.0.1");
    assert!(validate_sidecar_config(&cfg).is_ok());

    cfg = sidecar_config_with_port(cfg, 18050);
    assert_eq!(cfg.port, 18050);

    cfg = sidecar_config_with_api_key(cfg, "secret_key".to_string());
    assert_eq!(cfg.api_key.as_deref(), Some("secret_key"));

    let mut invalid = cfg.clone();
    invalid.binary_path = "".to_string();
    assert!(matches!(
        validate_sidecar_config(&invalid),
        Err(SidecarError::InvalidConfig(_))
    ));

    invalid = cfg.clone();
    invalid.host = "192.168.0.1".to_string();
    assert!(matches!(
        validate_sidecar_config(&invalid),
        Err(SidecarError::InvalidConfig(_))
    ));

    invalid = cfg.clone();
    invalid.context_size = 0;
    assert!(matches!(
        validate_sidecar_config(&invalid),
        Err(SidecarError::InvalidConfig(_))
    ));
}

#[test]
fn test_build_sidecar_command() {
    let mut cfg = new_sidecar_config("test-server", "model.gguf");
    cfg.api_key = Some("test-token".to_string());
    cfg.threads = Some(4);
    cfg.extra_args = vec!["--verbose".to_string()];

    let cmd = build_sidecar_command(&cfg, 18080);
    let prog = cmd.get_program().to_string_lossy();
    assert_eq!(prog, "test-server");

    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert!(args.contains(&"--host".to_string()));
    assert!(args.contains(&"127.0.0.1".to_string()));
    assert!(args.contains(&"--port".to_string()));
    assert!(args.contains(&"18080".to_string()));
    assert!(args.contains(&"-m".to_string()));
    assert!(args.contains(&"model.gguf".to_string()));
    assert!(args.contains(&"-t".to_string()));
    assert!(args.contains(&"4".to_string()));
    assert!(args.contains(&"--verbose".to_string()));

    let envs: Vec<_> = cmd.get_envs().collect();
    let has_key = envs.iter().any(|(k, v)| {
        *k == "LLAMA_API_KEY"
            && v.map(|s| s.to_string_lossy() == "test-token")
                .unwrap_or(false)
    });
    assert!(has_key);
}

#[test]
fn test_stdin_watchdog_termination() {
    let mut cfg = new_sidecar_config("cat", "");
    cfg.pass_server_args = false;

    let mut proc = spawn_sidecar_process(&cfg).expect("failed to spawn cat");
    assert!(proc.pid > 0);
    assert!(proc.port > 0);
    assert!(is_sidecar_process_alive(&mut proc));
    assert!(is_process_alive(proc.pid));

    // Stdin watchdog: closing stdin delivers EOF, causing cat to terminate cleanly.
    assert!(close_stdin(&mut proc));

    let exit_status = wait_for_process_exit(&mut proc).expect("wait failed");

    assert!(exit_status.success());
    assert!(!is_sidecar_process_alive(&mut proc));
}

#[test]
fn test_sidecar_process_kill() {
    let mut cfg = new_sidecar_config("sleep", "");
    cfg.pass_server_args = false;
    cfg.extra_args = vec!["60".to_string()];

    let mut proc = spawn_sidecar_process(&cfg).expect("failed to spawn sleep");
    let pid = proc.pid;
    assert!(is_sidecar_process_alive(&mut proc));
    assert!(is_process_alive(pid));

    kill_sidecar_process(&mut proc).expect("failed to kill process");
    assert!(!is_sidecar_process_alive(&mut proc));
    assert!(!is_process_alive(pid));
}

#[test]
fn test_sidecar_status_conversions() {
    assert_eq!(
        sidecar_status_to_backend_status(&SidecarStatus::Stopped),
        BackendStatus::Stopped
    );
    assert_eq!(
        sidecar_status_to_backend_status(&SidecarStatus::Starting),
        BackendStatus::Starting
    );
    assert_eq!(
        sidecar_status_to_backend_status(&SidecarStatus::Running),
        BackendStatus::Ready
    );
    assert_eq!(
        sidecar_status_to_backend_status(&SidecarStatus::Stopping),
        BackendStatus::Stopped
    );
    assert_eq!(
        sidecar_status_to_backend_status(&SidecarStatus::Failed("oom".to_string())),
        BackendStatus::Error("oom".to_string())
    );
    assert!(!is_sidecar_running(&SidecarStatus::Stopped));
    assert!(is_sidecar_running(&SidecarStatus::Running));
}

struct MockBackend {
    status: BackendStatus,
}

impl AssistantBackend for MockBackend {
    fn status(&self) -> BackendStatus {
        self.status.clone()
    }
    fn start(&mut self, _config: &BackendConfig) -> Result<(), BackendError> {
        self.status = BackendStatus::Ready;
        Ok(())
    }
    fn stop(&mut self) -> Result<(), BackendError> {
        self.status = BackendStatus::Stopped;
        Ok(())
    }
    fn is_ready(&self) -> bool {
        self.status == BackendStatus::Ready
    }
}

#[test]
fn test_assistant_backend_lifecycle() {
    let mut backend = MockBackend {
        status: BackendStatus::Stopped,
    };
    assert_eq!(backend.status(), BackendStatus::Stopped);
    assert!(!backend.is_ready());

    let cfg = new_backend_config(
        std::path::PathBuf::from("/bin/true"),
        std::path::PathBuf::from("model.gguf"),
        "127.0.0.1".to_string(),
        18080,
    );

    backend.start(&cfg).unwrap();
    assert_eq!(backend.status(), BackendStatus::Ready);
    assert!(backend.is_ready());

    backend.stop().unwrap();
    assert_eq!(backend.status(), BackendStatus::Stopped);
    assert!(!backend.is_ready());
}
