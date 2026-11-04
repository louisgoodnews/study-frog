//! Diary entry persistence and indexing.

use std::collections::BTreeMap;

use rusqlite::{Transaction, params};
use studyfrog_core::civil::{Date, format_date};
use studyfrog_core::diary::RatingValue;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::DiaryDoc;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};

pub use crate::diary_query::{
    diary_entries_for_item, diary_entries_for_run, diary_entries_for_stack, list_all_diary_entries,
    list_diary_entries_by_day, load_diary_entry, search_diary_entries,
};

/// An indexed diary entry.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredDiaryEntry {
    /// ULID.
    pub uid: Uid,
    /// Vault-relative path.
    pub path: String,
    /// Civil date.
    pub date: Date,
    /// Display title.
    pub title: String,
    /// Study time in minutes.
    pub study_minutes: Option<u32>,
    /// Ratings by scale key.
    pub ratings: BTreeMap<String, RatingValue>,
    /// Markdown body content.
    pub content: String,
    /// File mtime in nanoseconds.
    pub mtime: i64,
    /// Linked runs.
    pub runs: Vec<Uid>,
    /// Linked stacks.
    pub stacks: Vec<String>,
    /// Linked items.
    pub items: Vec<Uid>,
    /// Linked exams.
    pub exams: Vec<String>,
    /// Tags.
    pub tags: Vec<String>,
}

/// Serialize ratings map to JSON string.
pub fn ratings_to_json(ratings: &BTreeMap<String, RatingValue>) -> String {
    let mut map = serde_json::Map::new();
    for (k, v) in ratings {
        let val = match v {
            RatingValue::Int(n) => serde_json::Value::Number((*n).into()),
            RatingValue::Text(s) => serde_json::Value::String(s.clone()),
        };
        map.insert(k.clone(), val);
    }
    serde_json::Value::Object(map).to_string()
}

/// Deserialize ratings map from JSON string.
pub fn ratings_from_json(json: &str) -> BTreeMap<String, RatingValue> {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json) else {
        return BTreeMap::new();
    };
    let Some(obj) = val.as_object() else {
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    for (k, v) in obj {
        if let Some(n) = v.as_i64() {
            out.insert(k.clone(), RatingValue::Int(n));
        } else if let Some(s) = v.as_str() {
            out.insert(k.clone(), RatingValue::Text(s.to_owned()));
        }
    }
    out
}

/// Write a parsed diary document and its link indices to SQLite.
pub fn write_diary_entry(
    tx: &Transaction,
    path: &str,
    mtime: i64,
    doc: &DiaryDoc,
) -> Result<(), StoreError> {
    let uid = format_ulid(doc.meta.uid);
    let date_str = format_date(doc.date);
    let ratings_json = ratings_to_json(&doc.ratings);

    tx.prepare_cached(
        "INSERT INTO diary_entries (uid, path, date, title, study_minutes, ratings_json, content, mtime)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .and_then(|mut s| {
        s.execute(params![
            uid,
            path,
            date_str,
            doc.meta.title,
            doc.study_minutes,
            ratings_json,
            doc.body,
            mtime,
        ])
    })
    .map_err(sql_err)?;

    for run in &doc.runs {
        tx.prepare_cached("INSERT INTO diary_runs (diary_uid, run_uid) VALUES (?1, ?2)")
            .and_then(|mut s| s.execute(params![uid, format_ulid(*run)]))
            .map_err(sql_err)?;
    }

    for stack in &doc.stacks {
        tx.prepare_cached("INSERT INTO diary_stacks (diary_uid, stack_ref) VALUES (?1, ?2)")
            .and_then(|mut s| s.execute(params![uid, stack]))
            .map_err(sql_err)?;
    }

    for item in &doc.items {
        tx.prepare_cached("INSERT INTO diary_items (diary_uid, item_uid) VALUES (?1, ?2)")
            .and_then(|mut s| s.execute(params![uid, format_ulid(*item)]))
            .map_err(sql_err)?;
    }

    for exam in &doc.exams {
        tx.prepare_cached("INSERT INTO diary_exams (diary_uid, exam_id) VALUES (?1, ?2)")
            .and_then(|mut s| s.execute(params![uid, exam]))
            .map_err(sql_err)?;
    }

    for tag in &doc.meta.tags {
        tx.prepare_cached("INSERT INTO diary_tags (diary_uid, tag) VALUES (?1, ?2)")
            .and_then(|mut s| s.execute(params![uid, tag]))
            .map_err(sql_err)?;
    }

    Ok(())
}
