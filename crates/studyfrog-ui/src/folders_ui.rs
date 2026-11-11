//! Pure update logic for the Folders view.

pub use crate::folders_nav::{handle_key, open_entry, selected_parent};
pub use crate::folders_tree::{compare_entries, entries_from_vault, tree_visible_rows};

use crate::effect::Effect;
use crate::folders_types::{FolderEffect, FoldersMsg, NewFolderForm};
use crate::state::{Notice, NoticeKind, State};

/// Update Folders state and produce effects.
pub fn update(mut state: State, msg: FoldersMsg) -> (State, Vec<Effect>) {
    match msg {
        FoldersMsg::Scan => (state, vec![Effect::Folders(FolderEffect::Scan)]),
        FoldersMsg::Scanned(Ok(entries)) => {
            state.folders.entries = entries;
            (state, Vec::new())
        }
        FoldersMsg::Scanned(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
        FoldersMsg::Select(path) => {
            state.folders.selected = Some(path);
            (state, Vec::new())
        }
        FoldersMsg::ToggleExpand(path) => {
            if state.folders.expanded.contains(&path) {
                state.folders.expanded.remove(&path);
            } else {
                state.folders.expanded.insert(path);
            }
            (state, Vec::new())
        }
        FoldersMsg::ExpandAll => {
            for entry in &state.folders.entries {
                if entry.is_folder {
                    state.folders.expanded.insert(entry.rel_path.clone());
                }
            }
            (state, Vec::new())
        }
        FoldersMsg::CollapseAll => {
            state.folders.expanded.clear();
            (state, Vec::new())
        }
        FoldersMsg::SetSort(order) => {
            state.folders.sort = order;
            (state, Vec::new())
        }
        FoldersMsg::StartNewNote => {
            let parent = selected_parent(&state.folders);
            let effect = Effect::Folders(FolderEffect::CreateNote {
                folder: parent,
                title: "Untitled".to_owned(),
            });
            (state, vec![effect])
        }
        FoldersMsg::StartNewFolder => {
            let parent = selected_parent(&state.folders);
            state.folders.new_folder = Some(NewFolderForm {
                parent,
                name: "New Folder".to_owned(),
                is_stack: false,
            });
            (state, Vec::new())
        }
        FoldersMsg::EditNewFolder(name, is_stack) => {
            if let Some(f) = &mut state.folders.new_folder {
                f.name = name;
                f.is_stack = is_stack;
            }
            (state, Vec::new())
        }
        FoldersMsg::CommitNewFolder => {
            if let Some(f) = state.folders.new_folder.take() {
                let name = f.name.trim().to_owned();
                if !name.is_empty() {
                    let effect = Effect::Folders(FolderEffect::CreateFolder {
                        parent: f.parent,
                        name,
                        is_stack: f.is_stack,
                    });
                    return (state, vec![effect]);
                }
            }
            (state, Vec::new())
        }
        FoldersMsg::CancelNewFolder => {
            state.folders.new_folder = None;
            (state, Vec::new())
        }
        FoldersMsg::StartRename(path) => {
            let name = path
                .rsplit_once('/')
                .map_or(path.as_str(), |(_, n)| n)
                .to_owned();
            state.folders.renaming = Some((path, name));
            (state, Vec::new())
        }
        FoldersMsg::EditRename(text) => {
            if let Some((_, n)) = &mut state.folders.renaming {
                *n = text;
            }
            (state, Vec::new())
        }
        FoldersMsg::CommitRename => {
            if let Some((src, new_name)) = state.folders.renaming.take() {
                let new_name = new_name.trim().to_owned();
                if !new_name.is_empty() {
                    let effect = Effect::Folders(FolderEffect::Rename { src, new_name });
                    return (state, vec![effect]);
                }
            }
            (state, Vec::new())
        }
        FoldersMsg::CancelRename => {
            state.folders.renaming = None;
            (state, Vec::new())
        }
        FoldersMsg::AskMove(src) => {
            state.folders.move_target = Some((src, String::new()));
            (state, Vec::new())
        }
        FoldersMsg::SetMoveTarget(target) => {
            if let Some((_, t)) = &mut state.folders.move_target {
                *t = target;
            }
            (state, Vec::new())
        }
        FoldersMsg::CommitMove => {
            if let Some((src, dst)) = state.folders.move_target.take() {
                let effect = Effect::Folders(FolderEffect::PlanMove {
                    src,
                    dst_folder: dst,
                });
                return (state, vec![effect]);
            }
            (state, Vec::new())
        }
        FoldersMsg::MovePlanReady(Ok(plan)) => {
            if plan.is_cross_stack {
                state.folders.confirm_move = Some(Box::new(plan));
                (state, Vec::new())
            } else {
                (
                    state,
                    vec![Effect::Folders(FolderEffect::ExecuteMove(Box::new(plan)))],
                )
            }
        }
        FoldersMsg::MovePlanReady(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
        FoldersMsg::ConfirmMove => {
            if let Some(plan) = state.folders.confirm_move.take() {
                return (
                    state,
                    vec![Effect::Folders(FolderEffect::ExecuteMove(plan))],
                );
            }
            (state, Vec::new())
        }
        FoldersMsg::CancelMove => {
            state.folders.confirm_move = None;
            (state, Vec::new())
        }
        FoldersMsg::AskDelete(path) => {
            (state, vec![Effect::Folders(FolderEffect::PlanDelete(path))])
        }
        FoldersMsg::DeletePlanReady(Ok(plan)) => {
            state.folders.confirm_delete = Some(Box::new(plan));
            (state, Vec::new())
        }
        FoldersMsg::DeletePlanReady(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
        FoldersMsg::ConfirmDelete => {
            if let Some(plan) = state.folders.confirm_delete.take() {
                return (
                    state,
                    vec![Effect::Folders(FolderEffect::ExecuteDelete(plan))],
                );
            }
            (state, Vec::new())
        }
        FoldersMsg::CancelDelete => {
            state.folders.confirm_delete = None;
            (state, Vec::new())
        }
        FoldersMsg::Open(path) => {
            if path.ends_with(".pdf") {
                return crate::pdf_ui::open(state, path, 0);
            }
            if let Some(item) = state
                .library
                .items
                .iter()
                .find(|i| i.row.path.as_str() == path)
            {
                let uid = item.uid;
                let (state, mut fx) = crate::nav::navigate(state, crate::state::Screen::Editor);
                fx.push(Effect::LoadDocument(uid));
                return (state, fx);
            }
            (state, Vec::new())
        }
        FoldersMsg::KeyAction(key) => {
            let fx = handle_key(&mut state, key);
            (state, fx)
        }
        FoldersMsg::OpDone(Ok(info)) => {
            state.vault = Some(info);
            (
                state,
                vec![Effect::Folders(FolderEffect::Scan), Effect::LoadLibrary],
            )
        }
        FoldersMsg::OpDone(Err(e)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: e,
            });
            (state, Vec::new())
        }
    }
}
