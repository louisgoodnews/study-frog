//! Executing stack mark and unmark operations against a real vault (TASK-0217).

use std::fs;

use studyfrog_service::create::{fresh_id_source, refresh, slug};
use studyfrog_service::stack_mark::{mark_as_stack, plan_unmark, undo_mark_change, unmark_stack};

use crate::exec::Worker;
use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::stack_mark_ui::StackMarkEffect;

/// Execute a stack mark effect.
pub fn execute(w: &mut Worker, effect: StackMarkEffect, now_ns: i64) -> Vec<Msg> {
    let Some(session) = &mut w.session else {
        return vec![Msg::Modal(ModalMsg::UnmarkPlanLoaded(Err(
            "no vault is open".to_owned(),
        )))];
    };

    match effect {
        StackMarkEffect::PlanUnmark(folder) => {
            let res = plan_unmark(session, &folder)
                .map(Box::new)
                .map_err(|e| e.to_string());
            vec![Msg::Modal(ModalMsg::UnmarkPlanLoaded(res))]
        }
        StackMarkEffect::Unmark(folder) => {
            let res = unmark_stack(session, &folder).map_err(|e| e.to_string());
            vec![Msg::Modal(ModalMsg::Unmarked(res))]
        }
        StackMarkEffect::Mark(folder) => {
            let now_ms = (now_ns / 1_000_000) as u64;
            let ids = match fresh_id_source(now_ms) {
                Ok(i) => i,
                Err(e) => return vec![Msg::Modal(ModalMsg::Marked(Err(e.to_string())))],
            };
            let res = mark_as_stack(session, &folder, ids).map_err(|e| e.to_string());
            vec![Msg::Modal(ModalMsg::Marked(res))]
        }
        StackMarkEffect::Undo(snapshot) => {
            let res = undo_mark_change(session, &snapshot).map_err(|e| e.to_string());
            vec![Msg::Modal(ModalMsg::MarkUndone(res))]
        }
        StackMarkEffect::CreateFolder { parent, name } => {
            let folder_name = slug(&name);
            let dir = match &parent {
                Some(p) => session.root.join(p).join(&folder_name),
                None => session.root.join(&folder_name),
            };
            let res = fs::create_dir_all(&dir)
                .map_err(|e| e.to_string())
                .and_then(|()| refresh(session).map_err(|e| e.to_string()))
                .map(|info| (info, name));
            vec![Msg::Modal(ModalMsg::CreatedFolder(res))]
        }
    }
}
