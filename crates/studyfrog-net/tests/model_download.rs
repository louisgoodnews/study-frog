#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration and unit tests for model download, resumption, and SHA-256 verification.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use studyfrog_net::model_download::{
    DownloadStatus, calculate_download_progress, compute_file_sha256_hex, compute_sha256_hex,
    download_resumable, format_range_header, get_download_progress, init_download_state,
    parse_content_range, update_download_bytes, verify_file_sha256, verify_sha256,
};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_file_path(name: &str) -> PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "studyfrog_test_{}_{}_{}.bin",
        std::process::id(),
        id,
        name
    ));
    dir
}

#[test]
fn test_format_range_header() {
    assert_eq!(format_range_header(0), "bytes=0-");
    assert_eq!(format_range_header(1024), "bytes=1024-");
    assert_eq!(format_range_header(987654321), "bytes=987654321-");
}

#[test]
fn test_parse_content_range() {
    assert_eq!(parse_content_range("bytes 100-499/500"), Some((100, 500)));
    assert_eq!(parse_content_range("BYTES 0-99/1000"), Some((0, 1000)));
    assert_eq!(parse_content_range("bytes: 50-100/200"), Some((50, 200)));
    assert_eq!(parse_content_range("100-499/500"), Some((100, 500)));
    assert_eq!(parse_content_range("bytes 100-499/*"), None);
    assert_eq!(parse_content_range("invalid-header"), None);
    assert_eq!(parse_content_range("bytes /500"), None);
}

#[test]
fn test_sha256_computation_and_verification() {
    let empty_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    assert_eq!(compute_sha256_hex(b""), empty_hash);
    assert!(verify_sha256(b"", empty_hash));
    assert!(verify_sha256(b"", &empty_hash.to_uppercase()));
    assert!(!verify_sha256(b"abc", empty_hash));

    let data = b"studyfrog model downloader";
    let hash = compute_sha256_hex(data);
    assert_eq!(hash.len(), 64);
    assert!(verify_sha256(data, &hash));
}

#[test]
fn test_file_sha256() {
    let path = temp_file_path("sha256");
    let content = b"large chunk of test data for file hashing";
    fs::write(&path, content).unwrap();

    let expected = compute_sha256_hex(content);
    let file_hash = compute_file_sha256_hex(&path).unwrap();
    assert_eq!(expected, file_hash);
    assert!(verify_file_sha256(&path, &expected).unwrap());

    let _ = fs::remove_file(path);
}

#[test]
fn test_download_progress_calculation() {
    let p1 = calculate_download_progress(0, 1000);
    assert_eq!(p1.downloaded_bytes, 0);
    assert_eq!(p1.total_bytes, 1000);
    assert_eq!(p1.percentage, 0.0);

    let p2 = calculate_download_progress(500, 1000);
    assert_eq!(p2.percentage, 50.0);

    let p3 = calculate_download_progress(1000, 1000);
    assert_eq!(p3.percentage, 100.0);

    let p4 = calculate_download_progress(1200, 1000);
    assert_eq!(p4.percentage, 100.0);

    let p5 = calculate_download_progress(0, 0);
    assert_eq!(p5.percentage, 0.0);
}

#[test]
fn test_download_state_lifecycle() {
    let mut state = init_download_state("qwen", "http://example.com/m.gguf", "/tmp/m.gguf", 1000);
    assert_eq!(state.status, DownloadStatus::Idle);
    assert_eq!(state.downloaded_bytes, 0);

    update_download_bytes(&mut state, 400);
    assert_eq!(state.status, DownloadStatus::Downloading);
    assert_eq!(state.downloaded_bytes, 400);
    assert_eq!(get_download_progress(&state).percentage, 40.0);

    update_download_bytes(&mut state, 1000);
    assert_eq!(state.status, DownloadStatus::Completed);
    assert_eq!(get_download_progress(&state).percentage, 100.0);
}

fn serve_download(payload: &'static [u8]) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        let (mut conn, _) = listener.accept().unwrap();
        let mut buf = [0u8; 2048];
        let n = conn.read(&mut buf).unwrap();
        let req = String::from_utf8_lossy(&buf[..n]);

        let range_header = req.lines().find(|l| l.to_lowercase().starts_with("range:"));
        if let Some(line) = range_header {
            let start: usize = line
                .split('=')
                .nth(1)
                .and_then(|v| v.trim_end_matches('-').parse().ok())
                .unwrap_or(0);
            let slice = &payload[start..];
            let end = payload.len() - 1;
            let total = payload.len();
            let header = format!(
                "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes {start}-{end}/{total}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                slice.len()
            );
            conn.write_all(header.as_bytes()).unwrap();
            conn.write_all(slice).unwrap();
        } else {
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                payload.len()
            );
            conn.write_all(header.as_bytes()).unwrap();
            conn.write_all(payload).unwrap();
        }
    });
    (format!("http://{addr}/model.gguf"), handle)
}

#[test]
fn test_download_resumable_fresh() {
    let payload = b"complete model weights binary content";
    let (url, server) = serve_download(payload);
    let dest = temp_file_path("fresh");
    let sha = compute_sha256_hex(payload);

    let mut recorded_bytes = 0;
    let res = download_resumable(&url, &dest, Some(&sha), |p| {
        recorded_bytes = p.downloaded_bytes;
    });

    server.join().unwrap();
    assert!(res.is_ok());
    assert_eq!(recorded_bytes, payload.len() as u64);
    assert_eq!(fs::read(&dest).unwrap(), payload);

    let _ = fs::remove_file(dest);
}

#[test]
fn test_download_resumable_resumption() {
    let payload = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let dest = temp_file_path("resume");
    // Pre-seed first 10 bytes
    fs::write(&dest, &payload[..10]).unwrap();

    let (url, server) = serve_download(payload);
    let sha = compute_sha256_hex(payload);

    let res = download_resumable(&url, &dest, Some(&sha), |_| {});

    server.join().unwrap();
    assert!(res.is_ok());
    assert_eq!(fs::read(&dest).unwrap(), payload);

    let _ = fs::remove_file(dest);
}

#[test]
fn test_download_resumable_checksum_mismatch() {
    let payload = b"model data";
    let (url, server) = serve_download(payload);
    let dest = temp_file_path("mismatch");
    let bad_sha = "0000000000000000000000000000000000000000000000000000000000000000";

    let res = download_resumable(&url, &dest, Some(bad_sha), |_| {});
    server.join().unwrap();
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("checksum mismatch"));
    assert!(!dest.exists(), "a file with the wrong digest is removed");
}
