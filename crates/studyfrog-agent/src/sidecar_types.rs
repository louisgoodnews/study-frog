//! Sidecar process configuration and status types (ADR 0009, v0.9.0 roadmap 4.5).
//!
//! Follows Invariant P1 (zero inherent impls) and P6 (<= 250 lines).

use std::fmt;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::assistant_backend::BackendStatus;

/// Current lifecycle status of the managed sidecar process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SidecarStatus {
    /// Not running and no child process held.
    #[default]
    Stopped,
    /// Spawning and waiting for health/port readiness.
    Starting,
    /// Actively running and ready to serve requests.
    Running,
    /// Graceful termination in progress.
    Stopping,
    /// Terminated or failed with error message.
    Failed(String),
}

/// Information about a running sidecar process handle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidecarProcessHandle {
    /// Process ID.
    pub pid: u32,
    /// Loopback port.
    pub port: u16,
}

/// Creates a new [`SidecarProcessHandle`].
pub fn new_sidecar_process_handle(pid: u32, port: u16) -> SidecarProcessHandle {
    SidecarProcessHandle { pid, port }
}

/// Port range specification for ephemeral binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRange {
    /// Minimum port in range (inclusive).
    pub min_port: u16,
    /// Maximum port in range (inclusive).
    pub max_port: u16,
}

/// Create a default port range for local sidecars (18000..=19000).
pub fn default_port_range() -> PortRange {
    PortRange {
        min_port: 18000,
        max_port: 19000,
    }
}

/// Constructs a valid [`PortRange`].
pub fn new_port_range(min_port: u16, max_port: u16) -> Result<PortRange, PortError> {
    if min_port == 0 || min_port > max_port {
        Err(PortError::InvalidRange(min_port, max_port))
    } else {
        Ok(PortRange { min_port, max_port })
    }
}

/// Checks whether a given port falls within a [`PortRange`].
pub fn port_range_contains(range: &PortRange, port: u16) -> bool {
    port >= range.min_port && port <= range.max_port
}

/// Plain configuration data for starting a managed sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidecarConfig {
    /// Binary executable path or name.
    pub binary_path: String,
    /// GGUF model path.
    pub model_path: String,
    /// Loopback host (must be 127.0.0.1 or localhost).
    pub host: String,
    /// Port (0 for ephemeral allocation).
    pub port: u16,
    /// Port range for scanning.
    pub port_range: PortRange,
    /// Context window size in tokens.
    pub context_size: u32,
    /// Inactivity timeout in seconds before auto-shutdown.
    pub idle_timeout_secs: u64,
    /// Ephemeral bearer token.
    pub api_key: Option<String>,
    /// CPU threads count.
    pub threads: Option<u32>,
    /// Whether to pass standard llama-server arguments.
    pub pass_server_args: bool,
    /// Additional arguments.
    pub extra_args: Vec<String>,
    /// Environment variables.
    pub env_vars: Vec<(String, String)>,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        default_sidecar_config()
    }
}

/// Create a new configuration for sidecar execution with given binary and model paths.
pub fn new_sidecar_config(binary_path: &str, model_path: &str) -> SidecarConfig {
    SidecarConfig {
        binary_path: binary_path.to_string(),
        model_path: model_path.to_string(),
        host: "127.0.0.1".to_string(),
        port: 0,
        port_range: default_port_range(),
        context_size: 4096,
        idle_timeout_secs: 300,
        api_key: None,
        threads: None,
        pass_server_args: true,
        extra_args: Vec::new(),
        env_vars: Vec::new(),
    }
}

/// Create a default configuration for sidecar execution.
pub fn default_sidecar_config() -> SidecarConfig {
    new_sidecar_config("llama-server", "")
}

/// Checks whether a hostname or IP string represents a loopback address.
pub fn is_loopback_host(host: &str) -> bool {
    if host == "localhost" || host == "127.0.0.1" || host == "::1" || host == "[::1]" {
        return true;
    }
    host.parse::<IpAddr>()
        .map(|ip| ip.is_loopback())
        .unwrap_or(false)
}

/// Validates that a [`SidecarConfig`] adheres to loopback security and invariant rules.
pub fn validate_sidecar_config(config: &SidecarConfig) -> Result<(), SidecarError> {
    if config.binary_path.trim().is_empty() {
        return Err(SidecarError::InvalidConfig("empty binary path".into()));
    }
    if !is_loopback_host(&config.host) {
        return Err(SidecarError::InvalidConfig("host is not loopback".into()));
    }
    if config.port_range.min_port == 0 || config.port_range.min_port > config.port_range.max_port {
        return Err(SidecarError::InvalidConfig("invalid port range".into()));
    }
    if config.context_size == 0 {
        return Err(SidecarError::InvalidConfig(
            "context size must be > 0".into(),
        ));
    }
    Ok(())
}

/// Produces a copy of [`SidecarConfig`] with an assigned port.
pub fn sidecar_config_with_port(mut config: SidecarConfig, port: u16) -> SidecarConfig {
    config.port = port;
    config
}

/// Produces a copy of [`SidecarConfig`] with an ephemeral API key.
pub fn sidecar_config_with_api_key(mut config: SidecarConfig, api_key: String) -> SidecarConfig {
    config.api_key = Some(api_key);
    config
}

/// Returns true if the sidecar status represents an active running process.
pub fn is_sidecar_running(status: &SidecarStatus) -> bool {
    matches!(status, SidecarStatus::Running)
}

/// Converts internal [`SidecarStatus`] into polymorphic [`BackendStatus`].
pub fn sidecar_status_to_backend_status(status: &SidecarStatus) -> BackendStatus {
    match status {
        SidecarStatus::Stopped | SidecarStatus::Stopping => BackendStatus::Stopped,
        SidecarStatus::Starting => BackendStatus::Starting,
        SidecarStatus::Running => BackendStatus::Ready,
        SidecarStatus::Failed(err) => BackendStatus::Error(err.clone()),
    }
}

/// Errors occurring during port selection and binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortError {
    /// Port range bounds are inverted or 0 (min, max).
    InvalidRange(u16, u16),
    /// No port available in range (min, max).
    NoPortAvailable(u16, u16),
    /// Bind call failed.
    BindFailed(String),
}

impl fmt::Display for PortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange(min, max) => write!(f, "invalid port range: {min} > {max} or 0"),
            Self::NoPortAvailable(min, max) => write!(f, "no port available in {min}..={max}"),
            Self::BindFailed(msg) => write!(f, "port binding failed: {msg}"),
        }
    }
}

impl std::error::Error for PortError {}

/// Errors occurring during sidecar process supervision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarError {
    /// Process failed to spawn.
    ProcessSpawnFailed(String),
    /// No port could be found.
    PortUnavailable(String),
    /// Process with pid not found.
    ProcessNotFound(u32),
    /// Process terminated unexpectedly.
    ProcessTerminated(String),
    /// Process executable does not match target.
    ProcessMismatch(String),
    /// Config failed validation.
    InvalidConfig(String),
    /// I/O error occurred.
    Io(String),
    /// Timed out waiting for operation.
    Timeout(String),
}

impl fmt::Display for SidecarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessSpawnFailed(m) => write!(f, "process spawn failed: {m}"),
            Self::PortUnavailable(m) => write!(f, "port unavailable: {m}"),
            Self::ProcessNotFound(pid) => write!(f, "process not found for pid {pid}"),
            Self::ProcessTerminated(m) => write!(f, "process terminated: {m}"),
            Self::ProcessMismatch(m) => write!(f, "process mismatch: {m}"),
            Self::InvalidConfig(m) => write!(f, "invalid config: {m}"),
            Self::Io(m) => write!(f, "sidecar io error: {m}"),
            Self::Timeout(m) => write!(f, "sidecar timeout: {m}"),
        }
    }
}

impl std::error::Error for SidecarError {}
