//! A sink that appends one line per record to a file the caller names, rotating it by size.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::format::format_line;
use crate::record::Record;
use crate::sink::Sink;
use crate::state::lock;

/// The state of one file sink. Plain data, owned by the sink's closure.
struct FileState {
    path: PathBuf,
    file: Option<File>,
    size: u64,
    max_bytes: u64,
    keep: usize,
}

/// Creates a sink that appends each record as one line to `path`, creating parent folders.
/// Before a line would take the file past `max_bytes`, the file is rotated: `path` becomes
/// `path.1`, older copies move up to `path.<keep>`, and the oldest copy is dropped. A line is
/// always written to an empty file, even if it is longer than `max_bytes`.
/// Write errors are ignored, because a log must not stop the program.
pub fn file_sink(path: &Path, max_bytes: u64, keep: usize) -> io::Result<Sink> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let file = open_append(path)?;
    let size = file.metadata()?.len();
    let state = Mutex::new(FileState {
        path: path.to_path_buf(),
        file: Some(file),
        size,
        max_bytes,
        keep,
    });
    Ok(Box::new(move |record: &Record| {
        let line = format_line(record);
        let mut guard = lock(&state);
        let _ = write_line(&mut guard, &line);
    }))
}

fn write_line(state: &mut FileState, line: &str) -> io::Result<()> {
    let needed = u64::try_from(line.len() + 1).unwrap_or(u64::MAX);
    if state.size > 0 && state.size.saturating_add(needed) > state.max_bytes {
        rotate(state)?;
    }
    if state.file.is_none() {
        state.file = Some(open_append(&state.path)?);
    }
    if let Some(file) = state.file.as_mut() {
        writeln!(file, "{line}")?;
        state.size = state.size.saturating_add(needed);
    }
    Ok(())
}

/// Shifts the numbered copies up by one and starts a new file. Renames are best effort: a missing
/// copy is not an error, and the new file is created either way.
fn rotate(state: &mut FileState) -> io::Result<()> {
    // Close the open handle first, so that the file can be renamed on every platform.
    state.file = None;
    if state.keep == 0 {
        let _ = fs::remove_file(&state.path);
    } else {
        let _ = fs::remove_file(numbered(&state.path, state.keep));
        for index in (1..state.keep).rev() {
            let _ = fs::rename(
                numbered(&state.path, index),
                numbered(&state.path, index + 1),
            );
        }
        let _ = fs::rename(&state.path, numbered(&state.path, 1));
    }
    state.size = 0;
    state.file = Some(open_append(&state.path)?);
    Ok(())
}

/// The path with `.<index>` appended, for example `studyfrog.log.2`.
fn numbered(path: &Path, index: usize) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(format!(".{index}"));
    PathBuf::from(name)
}

fn open_append(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}
