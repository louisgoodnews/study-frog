//! The parts of the control snapshot that describe one screen: the item page, the open dialog,
//! the rehearsal and the library list. Pure.

use serde_json::{Value, json};
use studyfrog_core::plan::item_kind_name;
use studyfrog_core::run::{run_mode_name, run_state_name};
use studyfrog_core::status::status_name;
use studyfrog_rehearsal::session::{ItemPhase, current_item};

use crate::control_state::ulid;
use crate::editor::{Editor, is_dirty};
use crate::modal_types::Modal;
use crate::run_state::RunView;
use crate::setup::Preview;
use crate::state::{Screen, State};

/// Rows of the library list a snapshot carries at most.
pub const MAX_ROWS: usize = 100;

/// The item page.
pub fn editor_json(e: &Editor) -> Value {
    let fields: Vec<Value> = e
        .form
        .fields
        .iter()
        .enumerate()
        .map(|(i, f)| json!({"index": i, "label": f.label, "text": f.text}))
        .collect();
    json!({
        "uid": ulid(e.uid), "path": e.path, "title": e.form.title, "tags": e.form.tags,
        "color": e.form.color, "fields": fields, "dirty": is_dirty(e), "saving": e.saving.is_some(),
        "conflict": e.conflict, "error": e.fail, "confirm_delete": e.confirm_delete,
    })
}

/// The open dialog.
pub fn modal_json(m: &Modal) -> Value {
    match m {
        Modal::Create(f) => json!({
            "kind": "create", "item_kind": format!("{:?}", f.kind).to_lowercase(),
            "title": f.title, "parent": f.parent, "error": f.error, "busy": f.busy,
        }),
        Modal::Vault => json!({"kind": "vault"}),
        Modal::Import => json!({"kind": "import"}),
        Modal::Transfer => json!({"kind": "transfer"}),
        Modal::ConfirmUnmark(_) => json!({"kind": "confirm_unmark"}),
        Modal::PdfExport(_) => json!({"kind": "pdf_export"}),
    }
}

fn phase_name(p: ItemPhase) -> &'static str {
    match p {
        ItemPhase::NoItem => "no_item",
        ItemPhase::Shown => "shown",
        ItemPhase::Revealed => "revealed",
    }
}

fn run_json(r: &RunView) -> Value {
    let s = &r.session;
    let current = current_item(s);
    let content = current.and_then(|u| r.content.get(&u));
    let revealed = s.phase == ItemPhase::Revealed;
    let scored = r
        .scored
        .as_ref()
        .map(|x| json!({"rating": format!("{:?}", x.rating).to_lowercase(), "engine": x.engine}));
    json!({
        "uid": ulid(s.run.uid), "state": run_state_name(s.run.state), "mode": run_mode_name(s.run.mode),
        "position": s.cursor.min(s.queue.len()), "queue": s.queue.len(), "phase": phase_name(s.phase),
        "paused": s.paused_at_ms.is_some(), "item": current.map(ulid),
        "title": content.map(|c| c.title.clone()), "prompt": content.map(|c| c.prompt.clone()),
        "answer": content.filter(|_| revealed).map(|c| c.answer.clone()),
        "typed": r.answer, "scored": scored, "events": r.events.len(),
        "exam": crate::control_exam::exam_json(r),
    })
}

fn preview_json(p: &Preview) -> Value {
    match p {
        Preview::Unknown => json!({"status": "unknown"}),
        Preview::Counting => json!({"status": "counting"}),
        Preview::Count(n) => json!({"status": "count", "items": n}),
        Preview::Invalid(why) => json!({"status": "invalid", "reason": why}),
        Preview::Failed(why) => json!({"status": "failed", "reason": why}),
    }
}

/// The rehearse screen: the setup and the run on screen.
pub fn rehearse_json(state: &State) -> Value {
    let r = &state.rehearse;
    if state.screen != Screen::Rehearse && r.run.is_none() {
        return Value::Null;
    }
    json!({
        "step": format!("{:?}", r.step).to_lowercase(),
        "mode": run_mode_name(r.form.mode),
        "preview": preview_json(&r.preview),
        "starting": r.starting,
        "run": r.run.as_ref().map(run_json),
    })
}

/// The rows a library query lets through, at most `limit`.
pub fn shown_rows(state: &State, limit: usize) -> Vec<Value> {
    let lib = &state.library;
    lib.shown
        .iter()
        .filter_map(|&i| lib.items.get(i))
        .take(limit)
        .map(|it| {
            let r = &it.row;
            json!({"uid": ulid(it.uid), "key": r.key, "title": r.title, "kind": item_kind_name(r.kind),
                   "status": status_name(r.status), "path": r.path, "tags": r.tags, "color": it.color})
        })
        .collect()
}

/// The library list, when it is on screen.
pub fn library_json(state: &State) -> Value {
    if state.screen != Screen::Library {
        return Value::Null;
    }
    let lib = &state.library;
    let error = lib
        .query_error
        .as_ref()
        .map(|e| json!({"column": e.column, "message": e.message}));
    json!({
        "query": lib.search.query, "query_error": error, "total": lib.items.len(),
        "in_scope": lib.scope_total, "shown": lib.shown.len(), "rows": shown_rows(state, MAX_ROWS),
    })
}
