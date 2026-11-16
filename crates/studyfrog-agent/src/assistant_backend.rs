//! Lifecycle trait and status types for assistant backends (Section 4.5).
//!
//! Enforces Invariant P1: zero inherent impl blocks for domain types.
//! Polymorphism across backends is handled via [`AssistantBackend`].

use std::fmt;
use std::path::PathBuf;

/// Operating status of an assistant backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendStatus {
    /// Backend process is stopped / not running.
    Stopped,
    /// Backend process is starting up or loading models.
    Starting,
    /// Backend is running and ready to handle requests.
    Ready,
    /// Backend encountered a fatal error.
    Error(String),
}

/// Errors occurring during assistant backend lifecycle transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    /// Backend process is already active.
    AlreadyRunning,
    /// Backend process is not currently running.
    NotRunning,
    /// Failed to spawn the backend process.
    SpawnFailed(String),
    /// Failed to stop the backend process.
    StopFailed(String),
    /// Health check or readiness check failed.
    HealthCheckFailed(String),
    /// Configuration validation error.
    ConfigError(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRunning => write!(f, "backend is already running"),
            Self::NotRunning => write!(f, "backend is not running"),
            Self::SpawnFailed(e) => write!(f, "failed to spawn backend: {e}"),
            Self::StopFailed(e) => write!(f, "failed to stop backend: {e}"),
            Self::HealthCheckFailed(e) => write!(f, "backend health check failed: {e}"),
            Self::ConfigError(e) => write!(f, "backend configuration error: {e}"),
        }
    }
}

impl std::error::Error for BackendError {}

/// Configuration for an assistant backend instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendConfig {
    /// Path to the sidecar executable.
    pub binary_path: PathBuf,
    /// Path to the model file (.gguf).
    pub model_path: PathBuf,
    /// Host to bind to (strictly loopback).
    pub host: String,
    /// Port to bind to.
    pub port: u16,
    /// Ephemeral authentication key.
    pub api_key: Option<String>,
    /// Model context size in tokens.
    pub context_size: u32,
    /// Number of CPU compute threads.
    pub threads: Option<u32>,
}

/// Creates a new [`BackendConfig`] with sensible defaults.
pub fn new_backend_config(
    binary_path: PathBuf,
    model_path: PathBuf,
    host: String,
    port: u16,
) -> BackendConfig {
    BackendConfig {
        binary_path,
        model_path,
        host,
        port,
        api_key: None,
        context_size: 2048,
        threads: None,
    }
}

/// Lifecycle management trait for assistant backends (local sidecar or daemon).
pub trait AssistantBackend: Send + Sync {
    /// Returns the current lifecycle status of the backend.
    fn status(&self) -> BackendStatus;

    /// Starts the backend with the provided configuration.
    fn start(&mut self, config: &BackendConfig) -> Result<(), BackendError>;

    /// Stops the backend and reclaims resources.
    fn stop(&mut self) -> Result<(), BackendError>;

    /// Returns true if the backend is ready to accept requests.
    fn is_ready(&self) -> bool;
}
