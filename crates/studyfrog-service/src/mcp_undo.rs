//! Undo entries of the structural changes an AI client made over MCP (STORY-0119): every move or
//! delete keeps its full snapshot (files, contents and dropped schedules) as JSON in the vault's
//! `.studyfrog/mcp-undo/` folder until the user undoes it or clears it.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use studyfrog_core::memory::{MemState, Sm2State};
use studyfrog_core::time::UnixMs;
use studyfrog_md::ulid::{format_ulid, parse_ulid};
use studyfrog_store::schedule::{StoredMem, StoredSm2};

use crate::folder_ops::{FolderSnapshot, UndoAction, undo_folder_op};
use crate::session::{ServiceError, VaultInfo, VaultSession};

/// A change that can be undone.
#[derive(Debug, Clone, PartialEq)]
pub struct UndoEntry {
    /// Its id (a ULID, the file name).
    pub id: String,
    /// What was done, in a sentence.
    pub label: String,
    /// When, in Unix milliseconds.
    pub at_ms: i64,
    /// How to take it back.
    pub snapshot: FolderSnapshot,
}

/// The folder of the undo entries of the vault at `root`.
pub fn undo_dir(root: &Path) -> PathBuf {
    root.join(".studyfrog").join("mcp-undo")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok())
        .collect()
}

fn action_json(a: &UndoAction) -> Value {
    match a {
        UndoAction::DeleteDir(p) => json!({"do": "delete_dir", "path": p}),
        UndoAction::DeleteFile(p) => json!({"do": "delete_file", "path": p}),
        UndoAction::MovePath { from, to } => json!({"do": "move", "from": from, "to": to}),
        UndoAction::WriteFile(p, t) => json!({"do": "write", "path": p, "text": t}),
        UndoAction::RecreateFile(p, b) => json!({"do": "recreate", "path": p, "hex": hex(b)}),
        UndoAction::RestoreMemory {
            item_uid,
            stack_uid,
            mem,
            sm2,
        } => json!({
            "do": "memory", "item": format_ulid(*item_uid), "stack": format_ulid(*stack_uid),
            "fsrs": mem.map(|m| json!([m.state.stability, m.state.difficulty, m.due.0, m.last_review.0])),
            "sm2": sm2.map(|s| json!([s.state.ease, s.state.repetitions, s.state.interval_days, s.due.0, s.last_review.0])),
        }),
    }
}

fn num(v: &Value, i: usize) -> Option<f64> {
    v.get(i).and_then(Value::as_f64)
}

fn int(v: &Value, i: usize) -> Option<i64> {
    v.get(i).and_then(Value::as_i64)
}

#[allow(clippy::cast_possible_truncation)]
fn mem_of(v: &Value) -> Option<StoredMem> {
    Some(StoredMem {
        state: MemState {
            stability: num(v, 0)? as f32,
            difficulty: num(v, 1)? as f32,
        },
        due: UnixMs(int(v, 2)?),
        last_review: UnixMs(int(v, 3)?),
    })
}

#[allow(clippy::cast_possible_truncation)]
fn sm2_of(v: &Value) -> Option<StoredSm2> {
    Some(StoredSm2 {
        state: Sm2State {
            ease: num(v, 0)? as f32,
            repetitions: u32::try_from(int(v, 1)?).ok()?,
            interval_days: u32::try_from(int(v, 2)?).ok()?,
        },
        due: UnixMs(int(v, 3)?),
        last_review: UnixMs(int(v, 4)?),
    })
}

fn action_of(v: &Value) -> Option<UndoAction> {
    let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
    Some(match v.get("do")?.as_str()? {
        "delete_dir" => UndoAction::DeleteDir(s("path")?),
        "delete_file" => UndoAction::DeleteFile(s("path")?),
        "move" => UndoAction::MovePath {
            from: s("from")?,
            to: s("to")?,
        },
        "write" => UndoAction::WriteFile(s("path")?, s("text")?),
        "recreate" => UndoAction::RecreateFile(s("path")?, unhex(&s("hex")?)?),
        "memory" => UndoAction::RestoreMemory {
            item_uid: parse_ulid(&s("item")?)?,
            stack_uid: parse_ulid(&s("stack")?)?,
            mem: v.get("fsrs").and_then(mem_of),
            sm2: v.get("sm2").and_then(sm2_of),
        },
        _ => return None,
    })
}

/// Keep an undo entry for a change that was just made.
pub fn record_undo(root: &Path, entry: &UndoEntry) -> Result<(), ServiceError> {
    let dir = undo_dir(root);
    let io = |e: std::io::Error| ServiceError::Io(format!("{}: {e}", dir.display()));
    fs::create_dir_all(&dir).map_err(io)?;
    let v = json!({
        "id": entry.id, "label": entry.label, "at_ms": entry.at_ms,
        "actions": entry.snapshot.0.iter().map(action_json).collect::<Vec<_>>(),
    });
    fs::write(dir.join(format!("{}.json", entry.id)), v.to_string()).map_err(io)
}

fn entry_of(text: &str) -> Option<UndoEntry> {
    let v: Value = serde_json::from_str(text).ok()?;
    let actions: Option<Vec<UndoAction>> = v
        .get("actions")?
        .as_array()?
        .iter()
        .map(action_of)
        .collect();
    Some(UndoEntry {
        id: v.get("id")?.as_str()?.to_owned(),
        label: v.get("label")?.as_str()?.to_owned(),
        at_ms: v.get("at_ms")?.as_i64()?,
        snapshot: FolderSnapshot(actions?),
    })
}

/// The undo entries of the vault, oldest first; damaged files are skipped.
pub fn list_undo(root: &Path) -> Vec<UndoEntry> {
    let Ok(dir) = fs::read_dir(undo_dir(root)) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = dir.filter_map(Result::ok).map(|e| e.path()).collect();
    paths.sort();
    paths
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok())
        .filter_map(|t| entry_of(&t))
        .collect()
}

/// Take back the change `id` and forget its entry.
pub fn undo_recorded(session: &mut VaultSession, id: &str) -> Result<VaultInfo, ServiceError> {
    let entry = list_undo(&session.root)
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| ServiceError::Invalid("that change can no longer be undone".to_owned()))?;
    let info = undo_folder_op(session, &entry.snapshot)?;
    let _ = fs::remove_file(undo_dir(&session.root).join(format!("{}.json", entry.id)));
    Ok(info)
}
