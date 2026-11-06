//! Performing the library's own effects: bulk edits, undo and the saved filters.

use studyfrog_core::time::UnixMs;
use studyfrog_service::bulk::{bulk_apply, bulk_undo};
use studyfrog_service::filters::{delete_filter, load_filters, save_filter};

use crate::exec::Worker;
use crate::library_bulk::{BulkMsg, LibraryEffect};
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::stack_view_ui::StackViewMsg;

fn msg(m: BulkMsg) -> Msg {
    Msg::Library(LibraryMsg::Bulk(m))
}

/// Perform one effect of the library and report what happened.
pub fn run(w: &mut Worker, effect: LibraryEffect, now_ns: i64) -> Msg {
    let now = UnixMs(now_ns / 1_000_000);
    let Some(s) = w.session.as_mut() else {
        return msg(BulkMsg::FiltersLoaded(Err("no vault is open".to_owned())));
    };
    match effect {
        LibraryEffect::LoadFilters => msg(BulkMsg::FiltersLoaded(Ok(load_filters(s)))),
        LibraryEffect::SaveFilter(f) => msg(BulkMsg::FiltersSaved(
            save_filter(s, f).map_err(|e| e.to_string()),
        )),
        LibraryEffect::DeleteFilter(name) => msg(BulkMsg::FiltersSaved(
            delete_filter(s, &name).map_err(|e| e.to_string()),
        )),
        LibraryEffect::Bulk(uids, edit) => msg(BulkMsg::Done(
            bulk_apply(s, &uids, &edit, now)
                .map(Box::new)
                .map_err(|e| e.to_string()),
        )),
        LibraryEffect::LoadSummary(folder, generation) => {
            let summary = studyfrog_service::stack_summary::stack_summary(&s.conn, &folder, now)
                .map(Box::new)
                .map_err(|e| e.to_string());
            Msg::Library(LibraryMsg::Stack(StackViewMsg::Loaded(
                folder, generation, summary,
            )))
        }
        LibraryEffect::Transfer(job) => crate::exec_transfer::run(w, job, now_ns),
        LibraryEffect::LoadGoals(folder, uid) => {
            let text = studyfrog_service::goals::load_goals(s, uid).map_err(|e| e.to_string());
            Msg::Library(LibraryMsg::Stack(StackViewMsg::GoalsLoaded(folder, text)))
        }
        LibraryEffect::SaveGoals(uid, text) => {
            let done = studyfrog_service::goals::save_goals(s, uid, &text, now)
                .map(Box::new)
                .map_err(|e| e.to_string());
            Msg::Library(LibraryMsg::Stack(StackViewMsg::GoalsSaved(done)))
        }
        LibraryEffect::SetStatus(uid, status) => {
            let done = studyfrog_service::set_status::set_status(s, uid, status, now)
                .map(Box::new)
                .map_err(|e| e.to_string());
            Msg::Library(LibraryMsg::Stack(StackViewMsg::StatusSet(done)))
        }
        LibraryEffect::MoveStack(src, dst_folder) => {
            let done = studyfrog_service::folder_ops::move_path(s, &src, &dst_folder)
                .map(|(info, _, plan)| Box::new((info, plan.src, plan.dst)))
                .map_err(|e| e.to_string());
            Msg::Library(LibraryMsg::Stack(StackViewMsg::Moved(done)))
        }
        LibraryEffect::Undo(snapshot) => msg(BulkMsg::Undone(
            bulk_undo(s, &snapshot)
                .map(Box::new)
                .map_err(|e| e.to_string()),
        )),
    }
}
