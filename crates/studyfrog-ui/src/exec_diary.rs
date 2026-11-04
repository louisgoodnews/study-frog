//! Execution of diary side effects against the filesystem and SQLite index.

use std::fs;

use studyfrog_core::civil::format_date;
use studyfrog_core::meta::Meta;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::doc::{DiaryDoc, Doc};
use studyfrog_md::render::render_doc;
use studyfrog_md::ulid::{compose_ulid, format_ulid};
use studyfrog_service::create::fresh_id_source;
use studyfrog_store::diary::{list_all_diary_entries, load_diary_entry};
use studyfrog_store::poll::prime;
use studyfrog_store::reindex::reindex;

use crate::diary_msg::DiaryMsg;
use crate::diary_types::{DiaryEditDraft, DiaryEffect};
use crate::exec::Worker;
use crate::msg::Msg;

fn parse_tags(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_stacks(s: &str) -> Vec<String> {
    s.split(',')
        .map(str::trim)
        .filter(|st| !st.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn determine_path(root: &std::path::Path, draft: &DiaryEditDraft, uid: Uid) -> String {
    if let Some(p) = &draft.path {
        return p.clone();
    }
    let base = format!("diary/{}.md", format_date(draft.date));
    let target = root.join(&base);
    if !target.exists() {
        return base;
    }
    if fs::read_to_string(&target).is_ok_and(|c| c.contains(&format_ulid(uid))) {
        return base;
    }
    for i in 1..100 {
        let suffixed = format!("diary/{}-{i}.md", format_date(draft.date));
        if !root.join(&suffixed).exists() {
            return suffixed;
        }
    }
    base
}

fn load_entries_msg(w: &Worker) -> Msg {
    let entries = match &w.session {
        Some(s) => list_all_diary_entries(&s.conn).unwrap_or_default(),
        None => Vec::new(),
    };
    Msg::Diary(DiaryMsg::EntriesLoaded(entries))
}

fn save_entry(w: &mut Worker, draft: DiaryEditDraft, now_ns: i64) -> Vec<Msg> {
    let Some(session) = &mut w.session else {
        return vec![Msg::Navigate(crate::state::Screen::Welcome)];
    };
    let now_ms = (now_ns / 1_000_000).max(0) as u64;
    let uid = match draft.uid {
        Some(u) => u,
        None => {
            let ids = fresh_id_source(now_ms).unwrap_or(studyfrog_service::create::IdSource {
                now_ms,
                random: [7; 10],
            });
            compose_ulid(ids.now_ms, ids.random)
        }
    };
    let path = determine_path(&session.root, &draft, uid);
    let diary_dir = session.root.join("diary");
    if let Err(e) = fs::create_dir_all(&diary_dir) {
        return vec![Msg::Diary(DiaryMsg::Saved(Err(format!(
            "Could not create diary directory: {e}"
        ))))];
    }
    let title_text = draft.title.trim().to_owned();
    let custom_title = if title_text.is_empty() {
        None
    } else {
        Some(title_text.clone())
    };
    let effective_title = custom_title
        .clone()
        .unwrap_or_else(|| format!("Diary: {}", format_date(draft.date)));
    let now_unix = UnixMs(now_ms as i64);
    let doc = DiaryDoc {
        meta: Meta {
            uid,
            title: effective_title,
            tags: parse_tags(&draft.tags),
            aliases: Vec::new(),
            created: Some(now_unix),
            updated: Some(now_unix),
        },
        date: draft.date,
        custom_title,
        study_minutes: draft.study_minutes,
        ratings: draft.ratings,
        runs: draft.runs,
        stacks: parse_stacks(&draft.stacks),
        items: draft.items,
        exams: draft.exams,
        body: draft.content,
        extra: Vec::new(),
    };
    let rendered = match render_doc(&Doc::Diary(doc)) {
        Ok(t) => t,
        Err(e) => {
            return vec![Msg::Diary(DiaryMsg::Saved(Err(format!(
                "Failed to render diary entry: {e}"
            ))))];
        }
    };
    let target = session.root.join(&path);
    if let Err(e) = fs::write(&target, rendered) {
        return vec![Msg::Diary(DiaryMsg::Saved(Err(format!(
            "Failed to write diary entry: {e}"
        ))))];
    }
    let _ = reindex(&mut session.conn, &session.root);
    session.poll = prime(&session.root).unwrap_or_default();
    let entries = list_all_diary_entries(&session.conn).unwrap_or_default();
    vec![
        Msg::Diary(DiaryMsg::EntriesLoaded(entries)),
        Msg::Diary(DiaryMsg::Saved(Ok("Diary entry saved.".to_owned()))),
    ]
}

fn delete_entry(w: &mut Worker, uid: Uid) -> Vec<Msg> {
    let Some(session) = &mut w.session else {
        return Vec::new();
    };
    let entry_opt = load_diary_entry(&session.conn, uid).ok().flatten();
    if let Some(entry) = entry_opt {
        let file_path = session.root.join(&entry.path);
        let _ = fs::remove_file(file_path);
        let _ = reindex(&mut session.conn, &session.root);
        session.poll = prime(&session.root).unwrap_or_default();
    }
    let entries = list_all_diary_entries(&session.conn).unwrap_or_default();
    vec![
        Msg::Diary(DiaryMsg::EntriesLoaded(entries)),
        Msg::Diary(DiaryMsg::Saved(Ok("Diary entry deleted.".to_owned()))),
    ]
}

/// Execute a diary side effect.
pub fn execute(w: &mut Worker, effect: DiaryEffect, now_ns: i64) -> Vec<Msg> {
    match effect {
        DiaryEffect::LoadEntries => vec![load_entries_msg(w)],
        DiaryEffect::SaveEntry(draft) => save_entry(w, *draft, now_ns),
        DiaryEffect::DeleteEntry(uid) => delete_entry(w, uid),
    }
}
