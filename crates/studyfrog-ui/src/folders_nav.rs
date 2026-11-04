//! Keyboard navigation, selection and entry opening for Folders view.

use crate::effect::Effect;
use crate::folders_tree::tree_visible_rows;
use crate::folders_types::{FolderEffect, FoldersKey, FoldersState};
use crate::state::State;

/// Open a note or PDF file.
pub fn open_entry(state: &State, path: &str) -> Vec<Effect> {
    if path.ends_with(".pdf") {
        return vec![Effect::Pdf(crate::pdf_types::PdfEffect::Library(
            crate::pdf_library_types::LibraryJob::Load,
        ))];
    }
    if path.ends_with(".md")
        && let Some(item) = state
            .library
            .items
            .iter()
            .find(|i| i.row.path.as_str() == path)
    {
        return vec![Effect::LoadDocument(item.uid)];
    }
    Vec::new()
}

/// Find the folder path containing the currently selected entry.
pub fn selected_parent(folders: &FoldersState) -> String {
    let Some(sel) = &folders.selected else {
        return String::new();
    };
    if let Some(entry) = folders.entries.iter().find(|e| &e.rel_path == sel) {
        if entry.is_folder {
            return entry.rel_path.clone();
        }
        return entry.parent.clone();
    }
    String::new()
}

/// Handle a keyboard navigation shortcut.
pub fn handle_key(state: &mut State, key: FoldersKey) -> Vec<Effect> {
    let visible = tree_visible_rows(&state.folders);
    if visible.is_empty() {
        return Vec::new();
    }
    let cur_idx = state
        .folders
        .selected
        .as_ref()
        .and_then(|sel| visible.iter().position(|r| &r.rel_path == sel));

    match key {
        FoldersKey::Up => {
            let next = cur_idx.map_or(0, |i| i.saturating_sub(1));
            state.folders.selected = Some(visible[next].rel_path.clone());
        }
        FoldersKey::Down => {
            let next = cur_idx.map_or(0, |i| (i + 1).min(visible.len() - 1));
            state.folders.selected = Some(visible[next].rel_path.clone());
        }
        FoldersKey::Right => {
            if let Some(idx) = cur_idx {
                let row = &visible[idx];
                if row.is_folder {
                    if !row.expanded {
                        state.folders.expanded.insert(row.rel_path.clone());
                    } else if idx + 1 < visible.len() && visible[idx + 1].depth > row.depth {
                        state.folders.selected = Some(visible[idx + 1].rel_path.clone());
                    }
                }
            }
        }
        FoldersKey::Left => {
            if let Some(idx) = cur_idx {
                let row = &visible[idx];
                if row.is_folder && row.expanded {
                    state.folders.expanded.remove(&row.rel_path);
                } else {
                    let parent = row.rel_path.rsplit_once('/').map_or("", |(p, _)| p);
                    if let Some(p_row) = visible.iter().find(|r| r.rel_path == parent) {
                        state.folders.selected = Some(p_row.rel_path.clone());
                    }
                }
            }
        }
        FoldersKey::Enter => {
            if let Some(idx) = cur_idx {
                let row = &visible[idx];
                if row.is_folder {
                    if row.expanded {
                        state.folders.expanded.remove(&row.rel_path);
                    } else {
                        state.folders.expanded.insert(row.rel_path.clone());
                    }
                } else {
                    return open_entry(state, &row.rel_path);
                }
            }
        }
        FoldersKey::F2 => {
            if let Some(sel) = &state.folders.selected {
                let name = sel.rsplit_once('/').map_or(sel.as_str(), |(_, n)| n);
                state.folders.renaming = Some((sel.clone(), name.to_owned()));
            }
        }
        FoldersKey::Delete => {
            if let Some(sel) = &state.folders.selected {
                return vec![Effect::Folders(FolderEffect::PlanDelete(sel.clone()))];
            }
        }
    }
    Vec::new()
}
