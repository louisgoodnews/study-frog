//! Atomic `.studyfrog/homework.toml` persistence and bidirectional SQLite synchronization.

use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use studyfrog_calendar::homework::{
    ChecklistItem, HomeworkItem, homework_priority_from_name, homework_priority_name,
    homework_status_from_name, homework_status_name, normalize_reminders, validate_homework,
};
use studyfrog_config::homework::{
    ChecklistItemDto, HOMEWORK_FILE, HomeworkFileDto, HomeworkItemDto, parse_homework_toml,
    render_homework_toml,
};
use studyfrog_core::civil::{LocalTime, format_date, format_minute, parse_date, parse_minute};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, io_err, sql_err};
use crate::homework::{delete_homework, list_homework, upsert_homework_tx};

/// Hidden directory inside a vault where index and homework files reside.
pub const INDEX_DIR: &str = ".studyfrog";

/// Path to `.studyfrog/homework.toml` under `root`.
pub fn homework_toml_path(root: &Path) -> PathBuf {
    root.join(INDEX_DIR).join(HOMEWORK_FILE)
}

fn config_err(path: &Path, message: impl Into<String>) -> StoreError {
    StoreError::Config {
        path: path.display().to_string(),
        message: message.into(),
    }
}

/// Convert a domain [`HomeworkItem`] into a [`HomeworkItemDto`].
pub fn homework_to_dto(item: &HomeworkItem) -> HomeworkItemDto {
    HomeworkItemDto {
        id: format_ulid(item.id),
        title: item.title.clone(),
        subject: item.subject.clone(),
        stack_uid: item.stack_uid.map(format_ulid),
        timetable_id: item.timetable_id.map(format_ulid),
        due_date: format_date(item.due.date),
        due_time: format_minute(item.due.minute),
        start_date: item.start_date.map(format_date),
        estimated_minutes: item.estimated_minutes,
        priority: homework_priority_name(item.priority).to_owned(),
        status: homework_status_name(item.status).to_owned(),
        notes: item.notes.clone(),
        reminders: normalize_reminders(&item.reminders),
        checklist: item
            .checklist
            .iter()
            .map(|c| ChecklistItemDto {
                text: c.text.clone(),
                done: c.done,
            })
            .collect(),
    }
}

/// Convert and validate a [`HomeworkItemDto`] into a domain [`HomeworkItem`].
pub fn dto_to_homework(dto: &HomeworkItemDto) -> Result<HomeworkItem, StoreError> {
    let parse_opt_uid = |v: Option<&str>, field: &str| {
        v.map(|s| parse_ulid(s).ok_or_else(|| StoreError::Corrupt(format!("{field}: {s:?}"))))
            .transpose()
    };
    let id =
        parse_ulid(&dto.id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {:?}", dto.id)))?;
    let due_date = parse_date(&dto.due_date)
        .ok_or_else(|| StoreError::Corrupt(format!("due_date: {:?}", dto.due_date)))?;
    let due_minute = parse_minute(&dto.due_time)
        .ok_or_else(|| StoreError::Corrupt(format!("due_time: {:?}", dto.due_time)))?;
    let start_date = dto
        .start_date
        .as_deref()
        .map(|s| parse_date(s).ok_or_else(|| StoreError::Corrupt(format!("start_date: {s:?}"))))
        .transpose()?;
    let priority = homework_priority_from_name(&dto.priority)
        .ok_or_else(|| StoreError::Corrupt(format!("priority: {:?}", dto.priority)))?;
    let status = homework_status_from_name(&dto.status)
        .ok_or_else(|| StoreError::Corrupt(format!("status: {:?}", dto.status)))?;
    let item = HomeworkItem {
        id,
        title: dto.title.clone(),
        subject: dto.subject.clone(),
        stack_uid: parse_opt_uid(dto.stack_uid.as_deref(), "stack_uid")?,
        timetable_id: parse_opt_uid(dto.timetable_id.as_deref(), "timetable_id")?,
        due: LocalTime {
            date: due_date,
            minute: due_minute,
        },
        start_date,
        estimated_minutes: dto.estimated_minutes,
        priority,
        status,
        checklist: dto
            .checklist
            .iter()
            .map(|c| ChecklistItem {
                text: c.text.clone(),
                done: c.done,
            })
            .collect(),
        notes: dto.notes.clone(),
        reminders: normalize_reminders(&dto.reminders),
    };
    validate_homework(&item).map_err(|e| StoreError::Corrupt(e.to_string()))?;
    Ok(item)
}

/// Read and validate `.studyfrog/homework.toml`. Returns `Ok(None)` when absent.
pub fn read_homework_toml(root: &Path) -> Result<Option<Vec<HomeworkItem>>, StoreError> {
    let path = homework_toml_path(root);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(io_err(&path, &e)),
    };
    let parsed = parse_homework_toml(&text).map_err(|e| config_err(&path, e.to_string()))?;
    let mut seen: HashSet<Uid> = HashSet::with_capacity(parsed.homework.len());
    let mut items = Vec::with_capacity(parsed.homework.len());
    for dto in &parsed.homework {
        let item = dto_to_homework(dto).map_err(|e| config_err(&path, e.to_string()))?;
        if !seen.insert(item.id) {
            return Err(config_err(
                &path,
                format!("duplicate homework id {}", dto.id),
            ));
        }
        items.push(item);
    }
    Ok(Some(items))
}

/// Atomically write `.studyfrog/homework.toml` using a temporary sibling file and rename.
pub fn write_homework_toml(root: &Path, items: &[HomeworkItem]) -> Result<(), StoreError> {
    let dir = root.join(INDEX_DIR);
    fs::create_dir_all(&dir).map_err(|e| io_err(&dir, &e))?;
    let target = dir.join(HOMEWORK_FILE);
    let temp = dir.join(format!(".{HOMEWORK_FILE}.tmp"));
    let dto = HomeworkFileDto {
        homework: items.iter().map(homework_to_dto).collect(),
    };
    fs::write(&temp, render_homework_toml(&dto)).map_err(|e| io_err(&temp, &e))?;
    fs::rename(&temp, &target).map_err(|e| io_err(&target, &e))
}

/// Export SQLite homework rows to `.studyfrog/homework.toml`.
pub fn sync_toml_from_sqlite(conn: &Connection, root: &Path) -> Result<usize, StoreError> {
    let items = list_homework(conn)?;
    write_homework_toml(root, &items)?;
    Ok(items.len())
}

/// Reconcile `.studyfrog/homework.toml` into SQLite (or backfill TOML from SQLite if absent post-restore).
pub fn sync_homework_from_toml(conn: &mut Connection, root: &Path) -> Result<usize, StoreError> {
    let Some(items) = read_homework_toml(root)? else {
        let existing = list_homework(conn)?;
        if existing.is_empty() {
            return Ok(0);
        }
        write_homework_toml(root, &existing)?;
        return Ok(existing.len());
    };
    let keep: HashSet<String> = items.iter().map(|i| format_ulid(i.id)).collect();
    let tx = conn.transaction().map_err(sql_err)?;
    for item in &items {
        upsert_homework_tx(&tx, item)?;
    }
    let existing_ids: Vec<String> = {
        let mut stmt = tx.prepare("SELECT id FROM homework").map_err(sql_err)?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(sql_err)?;
        rows.collect::<Result<Vec<_>, _>>().map_err(sql_err)?
    };
    for id in existing_ids {
        if !keep.contains(&id) {
            tx.execute("DELETE FROM homework WHERE id = ?1", [&id])
                .map_err(sql_err)?;
        }
    }
    tx.commit().map_err(sql_err)?;
    Ok(items.len())
}

/// Atomically upsert a homework item in SQLite and write `.studyfrog/homework.toml` before commit.
pub fn save_homework_synced(
    conn: &mut Connection,
    root: &Path,
    item: &HomeworkItem,
) -> Result<Vec<HomeworkItem>, StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    upsert_homework_tx(&tx, item)?;
    let all = list_homework(&tx)?;
    write_homework_toml(root, &all)?;
    tx.commit().map_err(sql_err)?;
    Ok(all)
}

/// Atomically delete a homework item in SQLite and rewrite `.studyfrog/homework.toml` before commit.
pub fn delete_homework_synced(
    conn: &mut Connection,
    root: &Path,
    id: studyfrog_core::uid::Uid,
) -> Result<Option<Vec<HomeworkItem>>, StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    if !delete_homework(&tx, id)? {
        return Ok(None);
    }
    let remaining = list_homework(&tx)?;
    write_homework_toml(root, &remaining)?;
    tx.commit().map_err(sql_err)?;
    Ok(Some(remaining))
}
