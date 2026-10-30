#![allow(dead_code, clippy::expect_used, clippy::unwrap_used)] // shared helpers: each test crate uses a subset

//! Test helpers: unique temporary directories without extra dependencies.

pub mod vault;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A directory removed again when dropped.
pub struct TempDir {
    pub path: PathBuf,
}

/// Create a fresh unique directory under the system temp dir.
pub fn temp_dir(label: &str) -> TempDir {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("studyfrog-{label}-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&path).expect("create temp dir");
    TempDir { path }
}

fn remove_dir(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

impl Drop for TempDir {
    fn drop(&mut self) {
        remove_dir(&self.path);
    }
}
