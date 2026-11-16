//! Resumable HTTP model downloader with SHA-256 integrity verification.
//!
//! Handles chunked downloading of large GGUF weights, HTTP `Range` resumption,
//! and streaming SHA-256 checksum computation.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;

use sha2::{Digest, Sha256};

/// Progress metrics for an ongoing or completed model download.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DownloadProgress {
    /// Number of bytes downloaded to disk.
    pub downloaded_bytes: u64,
    /// Total expected bytes of the target file.
    pub total_bytes: u64,
    /// Completion percentage clamped to `0.0..=100.0`.
    pub percentage: f32,
}

/// Lifecycle status of a model download.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadStatus {
    /// Initialized but not yet running.
    Idle,
    /// Actively receiving chunks over the network.
    Downloading,
    /// Temporarily paused.
    Paused,
    /// Fully downloaded and verified.
    Completed,
    /// Aborted or failed.
    Failed,
}

/// State of an in-progress or completed model download.
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadState {
    /// Model identifier.
    pub model_id: String,
    /// Remote URL.
    pub url: String,
    /// Destination path on local filesystem.
    pub target_path: String,
    /// Expected total size in bytes.
    pub total_bytes: u64,
    /// Bytes written to disk so far.
    pub downloaded_bytes: u64,
    /// Current download status.
    pub status: DownloadStatus,
    /// Error message if status is Failed.
    pub error: Option<String>,
}

/// Computes download progress given received and total bytes.
pub fn calculate_download_progress(downloaded_bytes: u64, total_bytes: u64) -> DownloadProgress {
    let percentage = if total_bytes == 0 {
        if downloaded_bytes > 0 { 100.0 } else { 0.0 }
    } else {
        ((downloaded_bytes as f64 / total_bytes as f64) * 100.0).clamp(0.0, 100.0) as f32
    };
    DownloadProgress {
        downloaded_bytes,
        total_bytes,
        percentage,
    }
}

/// Formats an HTTP `Range` header value starting from `start_byte`.
pub fn format_range_header(start_byte: u64) -> String {
    format!("bytes={start_byte}-")
}

/// Parses an HTTP `Content-Range` header value returning `Some((start_byte, total_bytes))`.
pub fn parse_content_range(header_val: &str) -> Option<(u64, u64)> {
    let s = header_val.trim();
    let s = s
        .strip_prefix("bytes ")
        .or_else(|| s.strip_prefix("BYTES "))
        .or_else(|| s.strip_prefix("bytes:"))
        .unwrap_or(s)
        .trim();
    let (range_str, total_str) = s.split_once('/')?;
    let (start_str, _) = range_str.split_once('-')?;
    Some((
        start_str.trim().parse().ok()?,
        total_str.trim().parse().ok()?,
    ))
}

fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            use std::fmt::Write;
            let _ = write!(&mut acc, "{b:02x}");
            acc
        })
}

/// Computes the hexadecimal SHA-256 digest of in-memory `data`.
pub fn compute_sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_string(&hasher.finalize())
}

/// Returns whether in-memory `data` matches `expected_hex` SHA-256 digest.
pub fn verify_sha256(data: &[u8], expected_hex: &str) -> bool {
    compute_sha256_hex(data).eq_ignore_ascii_case(expected_hex.trim())
}

/// Computes the hexadecimal SHA-256 digest of a file on disk.
pub fn compute_file_sha256_hex(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex_string(&hasher.finalize()))
}

/// Verifies whether the file at `path` matches `expected_hex` SHA-256 digest.
pub fn verify_file_sha256(path: &Path, expected_hex: &str) -> Result<bool, io::Error> {
    let actual = compute_file_sha256_hex(path)?;
    Ok(actual.eq_ignore_ascii_case(expected_hex.trim()))
}

/// Initializes a new download state.
pub fn init_download_state(id: &str, url: &str, target: &str, total_bytes: u64) -> DownloadState {
    DownloadState {
        model_id: id.to_string(),
        url: url.to_string(),
        target_path: target.to_string(),
        total_bytes,
        downloaded_bytes: 0,
        status: DownloadStatus::Idle,
        error: None,
    }
}

/// Updates bytes downloaded and recalculates status.
pub fn update_download_bytes(state: &mut DownloadState, downloaded: u64) {
    state.downloaded_bytes = downloaded;
    state.status = if state.total_bytes > 0 && downloaded >= state.total_bytes {
        DownloadStatus::Completed
    } else {
        DownloadStatus::Downloading
    };
}

/// Returns progress for the given download state.
pub fn get_download_progress(state: &DownloadState) -> DownloadProgress {
    calculate_download_progress(state.downloaded_bytes, state.total_bytes)
}

fn open_dest_for_status(
    status: u16,
    dest: &Path,
    content_range: Option<&str>,
    content_length: Option<&str>,
    existing_len: u64,
) -> Result<(File, u64, u64), String> {
    if status == 206 {
        let f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dest)
            .map_err(|e| format!("cannot open {dest:?}: {e}"))?;
        let total = content_range
            .and_then(parse_content_range)
            .map(|(_, tot)| tot)
            .unwrap_or(existing_len);
        Ok((f, existing_len, total))
    } else if (200..300).contains(&status) {
        let f = File::create(dest).map_err(|e| format!("cannot create {dest:?}: {e}"))?;
        let total = content_length
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        Ok((f, 0, total))
    } else {
        Err(format!("unexpected HTTP {status}"))
    }
}

/// Downloads a remote file to `dest` with HTTP Range resumption and SHA-256 verification.
pub fn download_resumable<F>(
    url: &str,
    dest: &Path,
    expected_sha256: Option<&str>,
    mut on_progress: F,
) -> Result<DownloadProgress, String>
where
    F: FnMut(DownloadProgress),
{
    let existing_len = fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .http_status_as_error(false)
        .build()
        .into();
    let mut req = agent.get(url);
    if existing_len > 0 {
        req = req.header("Range", &format_range_header(existing_len));
    }
    let mut resp = req.call().map_err(|e| format!("GET {url} failed: {e}"))?;
    let status = resp.status().as_u16();
    let content_range = resp
        .headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok());
    let content_length = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok());
    let (mut file, mut current, total) =
        open_dest_for_status(status, dest, content_range, content_length, existing_len)?;
    let mut reader = resp.body_mut().as_reader();
    let mut buf = [0u8; 32 * 1024];
    loop {
        let read = reader
            .read(&mut buf)
            .map_err(|e| format!("read error: {e}"))?;
        if read == 0 {
            break;
        }
        file.write_all(&buf[..read])
            .map_err(|e| format!("write error: {e}"))?;
        current += read as u64;
        on_progress(calculate_download_progress(current, total.max(current)));
    }
    file.flush().map_err(|e| format!("flush error: {e}"))?;
    if let Some(expected) = expected_sha256 {
        let ok = verify_file_sha256(dest, expected).map_err(|e| format!("checksum error: {e}"))?;
        if !ok {
            return Err(format!("checksum mismatch for {dest:?}"));
        }
    }
    Ok(calculate_download_progress(current, total.max(current)))
}
