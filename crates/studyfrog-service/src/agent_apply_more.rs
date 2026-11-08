//! Applying the diary entries and calendar sessions an AI client proposed (STORY-0119), once the
//! user accepted them or the MCP server was allowed to write.

use std::collections::BTreeMap;

use studyfrog_agent::wiki::{DiaryChange, SessionChange};
use studyfrog_calendar::session::{Ends, Repeat, Session, event_kind_from_name};
use studyfrog_core::civil::{LocalTime, parse_date};
use studyfrog_core::diary::RatingValue;
use studyfrog_md::doc::{DiaryDoc, Doc};
use studyfrog_md::render::render_doc;
use studyfrog_store::overview::load_overview;

use crate::create::{IdSource, meta, refresh, render_error, write_new};
use crate::planning::{new_session_id, save_session_checked};
use crate::session::{ServiceError, VaultSession};

fn bad(what: &str) -> ServiceError {
    ServiceError::Invalid(what.to_owned())
}

/// A diary file name for `date` that is not taken yet.
fn free_diary_path(session: &VaultSession, date: &str) -> String {
    let exists = |rel: &str| session.root.join(rel).exists();
    let first = format!("diary/{date}.md");
    if !exists(&first) {
        return first;
    }
    (1..1000)
        .map(|n| format!("diary/{date}-{n}.md"))
        .find(|p| !exists(p))
        .unwrap_or(first)
}

/// Write a new diary entry and index it. An existing entry of the day is never overwritten.
pub fn apply_diary(
    session: &mut VaultSession,
    d: &DiaryChange,
    ids: IdSource,
) -> Result<(), ServiceError> {
    let date = parse_date(&d.date).ok_or_else(|| bad("the diary date is not valid"))?;
    let custom = (!d.title.trim().is_empty()).then(|| d.title.trim().to_owned());
    let title = custom
        .clone()
        .unwrap_or_else(|| format!("Diary: {}", d.date));
    let mut meta = meta(&title, &[], ids);
    meta.updated = meta.created;
    let ratings: BTreeMap<String, RatingValue> = d
        .ratings
        .iter()
        .map(|(k, n)| (k.clone(), RatingValue::Int(i64::from(*n))))
        .collect();
    let doc = Doc::Diary(DiaryDoc {
        meta,
        date,
        custom_title: custom,
        study_minutes: d.minutes,
        ratings,
        runs: Vec::new(),
        stacks: d.stacks.clone(),
        items: Vec::new(),
        exams: Vec::new(),
        body: d.body.clone(),
        extra: Vec::new(),
    });
    let text = render_doc(&doc).map_err(render_error)?;
    let rel = free_diary_path(session, &d.date);
    write_new(&session.root.join(&rel), &text)?;
    refresh(session).map(|_| ())
}

/// Store a planned calendar entry (once, without reminders).
pub fn apply_session(
    session: &mut VaultSession,
    s: &SessionChange,
    ids: IdSource,
) -> Result<(), ServiceError> {
    let date = parse_date(&s.date).ok_or_else(|| bad("the session date is not valid"))?;
    let kind = event_kind_from_name(&s.kind).ok_or_else(|| bad("unknown kind of entry"))?;
    let stack = match &s.stack {
        None => None,
        Some(folder) => Some(
            load_overview(&session.conn)
                .map_err(ServiceError::Store)?
                .stacks
                .into_iter()
                .find(|st| &st.folder == folder)
                .map(|st| st.uid)
                .ok_or_else(|| bad("that stack no longer exists"))?,
        ),
    };
    let planned = Session {
        id: new_session_id(ids),
        title: s.title.clone(),
        stack,
        start: LocalTime {
            date,
            minute: s.start_minute,
        },
        duration_min: s.duration_min,
        repeat: Repeat::Never,
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: s.notes.clone(),
        kind,
        reminders: Vec::new(),
    };
    save_session_checked(session, planned).map(|_| ())
}
