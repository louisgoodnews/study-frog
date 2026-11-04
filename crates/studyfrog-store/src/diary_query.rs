//! Queries for indexed diary entries.

use rusqlite::{Connection, Row};
use studyfrog_core::civil::{Date, format_date, parse_date};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::diary::{StoredDiaryEntry, ratings_from_json};
use crate::error::{StoreError, sql_err};

fn row_to_entry(row: &Row) -> Result<StoredDiaryEntry, rusqlite::Error> {
    let uid_str: String = row.get(0)?;
    let path: String = row.get(1)?;
    let date_str: String = row.get(2)?;
    let title: String = row.get(3)?;
    let study_minutes: Option<u32> = row.get(4)?;
    let ratings_json: String = row.get(5)?;
    let content: String = row.get(6)?;
    let mtime: i64 = row.get(7)?;

    let uid = parse_ulid(&uid_str).unwrap_or(Uid(0));
    let date = parse_date(&date_str).unwrap_or(Date {
        year: 1970,
        month: 1,
        day: 1,
    });
    let ratings = ratings_from_json(&ratings_json);

    Ok(StoredDiaryEntry {
        uid,
        path,
        date,
        title,
        study_minutes,
        ratings,
        content,
        mtime,
        runs: Vec::new(),
        stacks: Vec::new(),
        items: Vec::new(),
        exams: Vec::new(),
        tags: Vec::new(),
    })
}

fn load_entry_links(conn: &Connection, entry: &mut StoredDiaryEntry) -> Result<(), StoreError> {
    let uid = format_ulid(entry.uid);

    let mut stmt = conn
        .prepare_cached("SELECT run_uid FROM diary_runs WHERE diary_uid = ?1 ORDER BY run_uid")
        .map_err(sql_err)?;
    let runs = stmt
        .query_map([&uid], |row| row.get::<_, String>(0))
        .map_err(sql_err)?
        .filter_map(|r| r.ok().and_then(|u| parse_ulid(&u)))
        .collect();
    entry.runs = runs;

    let mut stmt = conn
        .prepare_cached(
            "SELECT stack_ref FROM diary_stacks WHERE diary_uid = ?1 ORDER BY stack_ref",
        )
        .map_err(sql_err)?;
    let stacks = stmt
        .query_map([&uid], |row| row.get::<_, String>(0))
        .map_err(sql_err)?
        .filter_map(Result::ok)
        .collect();
    entry.stacks = stacks;

    let mut stmt = conn
        .prepare_cached("SELECT item_uid FROM diary_items WHERE diary_uid = ?1 ORDER BY item_uid")
        .map_err(sql_err)?;
    let items = stmt
        .query_map([&uid], |row| row.get::<_, String>(0))
        .map_err(sql_err)?
        .filter_map(|r| r.ok().and_then(|u| parse_ulid(&u)))
        .collect();
    entry.items = items;

    let mut stmt = conn
        .prepare_cached("SELECT exam_id FROM diary_exams WHERE diary_uid = ?1 ORDER BY exam_id")
        .map_err(sql_err)?;
    let exams = stmt
        .query_map([&uid], |row| row.get::<_, String>(0))
        .map_err(sql_err)?
        .filter_map(Result::ok)
        .collect();
    entry.exams = exams;

    let mut stmt = conn
        .prepare_cached("SELECT tag FROM diary_tags WHERE diary_uid = ?1 ORDER BY tag")
        .map_err(sql_err)?;
    let tags = stmt
        .query_map([&uid], |row| row.get::<_, String>(0))
        .map_err(sql_err)?
        .filter_map(Result::ok)
        .collect();
    entry.tags = tags;

    Ok(())
}

fn query_entries(
    conn: &Connection,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<StoredDiaryEntry>, StoreError> {
    let mut stmt = conn.prepare_cached(sql).map_err(sql_err)?;
    let base_entries: Vec<StoredDiaryEntry> = stmt
        .query_map(params, row_to_entry)
        .map_err(sql_err)?
        .filter_map(Result::ok)
        .collect();

    let mut entries = Vec::with_capacity(base_entries.len());
    for mut entry in base_entries {
        load_entry_links(conn, &mut entry)?;
        entries.push(entry);
    }
    Ok(entries)
}

/// List all diary entries for a given civil date.
pub fn list_diary_entries_by_day(
    conn: &Connection,
    date: Date,
) -> Result<Vec<StoredDiaryEntry>, StoreError> {
    let date_str = format_date(date);
    query_entries(
        conn,
        "SELECT uid, path, date, title, study_minutes, ratings_json, content, mtime
         FROM diary_entries WHERE date = ?1 ORDER BY path",
        &[&date_str],
    )
}

/// Search diary entries by text matching title or content.
pub fn search_diary_entries(
    conn: &Connection,
    query: &str,
) -> Result<Vec<StoredDiaryEntry>, StoreError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let pattern = format!("%{trimmed}%");
    query_entries(
        conn,
        "SELECT uid, path, date, title, study_minutes, ratings_json, content, mtime
         FROM diary_entries
         WHERE title LIKE ?1 OR content LIKE ?1
         ORDER BY date DESC, path",
        &[&pattern],
    )
}

/// Find all diary entries linked to an item.
pub fn diary_entries_for_item(
    conn: &Connection,
    item_uid: Uid,
) -> Result<Vec<StoredDiaryEntry>, StoreError> {
    let uid_str = format_ulid(item_uid);
    query_entries(
        conn,
        "SELECT d.uid, d.path, d.date, d.title, d.study_minutes, d.ratings_json, d.content, d.mtime
         FROM diary_entries d
         JOIN diary_items i ON i.diary_uid = d.uid
         WHERE i.item_uid = ?1
         ORDER BY d.date DESC, d.path",
        &[&uid_str],
    )
}

/// Find all diary entries linked to a stack reference.
pub fn diary_entries_for_stack(
    conn: &Connection,
    stack_ref: &str,
) -> Result<Vec<StoredDiaryEntry>, StoreError> {
    query_entries(
        conn,
        "SELECT d.uid, d.path, d.date, d.title, d.study_minutes, d.ratings_json, d.content, d.mtime
         FROM diary_entries d
         JOIN diary_stacks s ON s.diary_uid = d.uid
         WHERE s.stack_ref = ?1
         ORDER BY d.date DESC, d.path",
        &[&stack_ref],
    )
}

/// Find all diary entries linked to a rehearsal run.
pub fn diary_entries_for_run(
    conn: &Connection,
    run_uid: Uid,
) -> Result<Vec<StoredDiaryEntry>, StoreError> {
    let uid_str = format_ulid(run_uid);
    query_entries(
        conn,
        "SELECT d.uid, d.path, d.date, d.title, d.study_minutes, d.ratings_json, d.content, d.mtime
         FROM diary_entries d
         JOIN diary_runs r ON r.diary_uid = d.uid
         WHERE r.run_uid = ?1
         ORDER BY d.date DESC, d.path",
        &[&uid_str],
    )
}

/// Load a specific diary entry by its UID.
pub fn load_diary_entry(
    conn: &Connection,
    uid: Uid,
) -> Result<Option<StoredDiaryEntry>, StoreError> {
    let uid_str = format_ulid(uid);
    let mut entries = query_entries(
        conn,
        "SELECT uid, path, date, title, study_minutes, ratings_json, content, mtime
         FROM diary_entries WHERE uid = ?1",
        &[&uid_str],
    )?;
    Ok(entries.pop())
}

/// List all diary entries ordered chronologically.
pub fn list_all_diary_entries(conn: &Connection) -> Result<Vec<StoredDiaryEntry>, StoreError> {
    query_entries(
        conn,
        "SELECT uid, path, date, title, study_minutes, ratings_json, content, mtime
         FROM diary_entries ORDER BY date DESC, path",
        &[],
    )
}
