//! `logs` over the control protocol: the last entries of `console.jsonl`, filtered by level, and
//! with `--follow` every new entry as an event line, until the connection goes away.

use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value;
use studyfrog_service::logging::tail_lines;

use crate::cli::read_new;
use crate::control_cmd::LogsQuery;
use crate::control_reply::event_line;

/// Where answers and events of one connection go; shared by its reader and its log follower.
pub type SharedOut = Arc<Mutex<Box<dyn Write + Send>>>;

/// Lines read from the end of the log before filtering by level.
const SCAN_LINES: usize = 10_000;

fn rank(level: &str) -> u8 {
    match level {
        "error" => 2,
        "warn" => 1,
        _ => 0,
    }
}

/// Whether an entry is at least as serious as `min`.
pub fn passes(entry: &Value, min: Option<&str>) -> bool {
    let level = entry.get("level").and_then(Value::as_str).unwrap_or("info");
    min.is_none_or(|m| rank(level) >= rank(m))
}

/// The entries among `lines` that pass the level, as JSON, keeping the last `tail`.
pub fn select(lines: &[String], q: &LogsQuery) -> Vec<Value> {
    let picked: Vec<Value> = lines
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| passes(v, q.level.as_deref()))
        .collect();
    let from = picked.len().saturating_sub(q.tail);
    picked[from..].to_vec()
}

/// The last entries of the log at `path` that the query asks for.
pub fn read_entries(path: &Path, q: &LogsQuery) -> io::Result<Vec<Value>> {
    Ok(select(&tail_lines(path, SCAN_LINES.max(q.tail))?, q))
}

/// Write one line and flush; an error means the other side is gone.
pub fn write_line(out: &SharedOut, line: &str) -> io::Result<()> {
    let mut w = out
        .lock()
        .map_err(|_| io::Error::other("the output is poisoned"))?;
    writeln!(w, "{line}")?;
    w.flush()
}

/// Send the complete lines of `added` (after what was left over last time) that pass the level;
/// what follows the last line break is kept in `carry`.
fn send_new(out: &SharedOut, carry: &mut String, added: &[u8], q: &LogsQuery) -> io::Result<()> {
    carry.push_str(&String::from_utf8_lossy(added));
    let Some(end) = carry.rfind('\n') else {
        return Ok(());
    };
    let complete: String = carry.drain(..=end).collect();
    let lines: Vec<String> = complete.lines().map(str::to_owned).collect();
    let all = LogsQuery {
        tail: usize::MAX,
        ..q.clone()
    };
    for entry in select(&lines, &all) {
        write_line(out, &event_line("log", entry))?;
    }
    Ok(())
}

/// Send every entry written to `path` after its first `seen` bytes, until `alive` turns false or a write fails.
/// `wait` is the pause between looks at the file.
pub fn follow(
    path: &Path,
    mut seen: u64,
    q: &LogsQuery,
    out: &SharedOut,
    alive: &AtomicBool,
    wait: fn(),
) {
    let mut carry = String::new();
    while alive.load(Ordering::Relaxed) {
        wait();
        let Ok((added, end)) = read_new(path, seen) else {
            continue;
        };
        if end < seen {
            carry.clear();
        }
        seen = end;
        if send_new(out, &mut carry, &added, q).is_err() {
            return;
        }
    }
}
