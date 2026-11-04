//! Tree computation, flattening and sorting for the Folders view.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use studyfrog_service::library_rows::LibraryItem;

use crate::folders_types::{EntryKind, FolderEntry, FoldersState, SortOrder, VisibleRow};
use crate::state::VaultInfo;

/// Compare entries according to sort order (folders always come before files).
pub fn compare_entries(a: &FolderEntry, b: &FolderEntry, sort: SortOrder) -> Ordering {
    if a.is_folder != b.is_folder {
        return if a.is_folder {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    match sort {
        SortOrder::NameAsc => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        SortOrder::NameDesc => b.name.to_lowercase().cmp(&a.name.to_lowercase()),
        SortOrder::MtimeDesc => b.mtime_ns.cmp(&a.mtime_ns),
        SortOrder::MtimeAsc => a.mtime_ns.cmp(&b.mtime_ns),
    }
}

/// Compute visible rows in depth-first order respecting expanded folders and sort.
pub fn tree_visible_rows(folders: &FoldersState) -> Vec<VisibleRow> {
    let mut out = Vec::new();
    append_children(folders, "", 0, &mut out);
    out
}

fn append_children(folders: &FoldersState, parent: &str, depth: usize, out: &mut Vec<VisibleRow>) {
    let mut children: Vec<&FolderEntry> = folders
        .entries
        .iter()
        .filter(|e| e.parent == parent)
        .collect();
    children.sort_by(|a, b| compare_entries(a, b, folders.sort));

    for entry in children {
        let expanded = folders.expanded.contains(&entry.rel_path);
        let display_name = if entry.kind == EntryKind::Markdown {
            entry
                .name
                .strip_suffix(".md")
                .unwrap_or(&entry.name)
                .to_owned()
        } else {
            entry.name.clone()
        };
        out.push(VisibleRow {
            rel_path: entry.rel_path.clone(),
            display_name,
            is_folder: entry.is_folder,
            is_stack: entry.is_stack,
            depth,
            kind: entry.kind.clone(),
            expanded,
        });
        if entry.is_folder && expanded {
            append_children(folders, &entry.rel_path, depth + 1, out);
        }
    }
}

/// Synthesize folder entries from vault overview and library items when no scan is loaded.
pub fn entries_from_vault(vault: &VaultInfo, items: &[LibraryItem]) -> Vec<FolderEntry> {
    let mut entries = Vec::new();
    let mut folders_seen = BTreeSet::new();

    for stack in &vault.overview.stacks {
        if stack.folder.is_empty() {
            continue;
        }
        let parts: Vec<&str> = stack.folder.split('/').collect();
        let mut curr = String::new();
        for (i, part) in parts.iter().enumerate() {
            let parent = curr.clone();
            if !curr.is_empty() {
                curr.push('/');
            }
            curr.push_str(part);
            if folders_seen.insert(curr.clone()) {
                let is_stack = curr == stack.folder;
                entries.push(FolderEntry {
                    rel_path: curr.clone(),
                    name: (*part).to_owned(),
                    is_folder: true,
                    is_stack,
                    parent,
                    depth: i,
                    kind: EntryKind::Folder,
                    mtime_ns: 0,
                });
            }
        }
    }

    for item in items {
        let path = item.row.path.as_str();
        if path.is_empty() || path.ends_with("_stack.md") {
            continue;
        }
        let (parent, name) = path.rsplit_once('/').map_or(("", path), |(p, n)| (p, n));
        let depth = if parent.is_empty() {
            0
        } else {
            parent.split('/').count()
        };
        let kind = if name.ends_with(".pdf") {
            EntryKind::Pdf
        } else if name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") {
            EntryKind::Image
        } else if name.ends_with(".md") {
            EntryKind::Markdown
        } else {
            let ext = name
                .rsplit_once('.')
                .map_or("DOC", |(_, e)| e)
                .to_ascii_uppercase();
            EntryKind::Other(ext)
        };
        entries.push(FolderEntry {
            rel_path: path.to_owned(),
            name: name.to_owned(),
            is_folder: false,
            is_stack: false,
            parent: parent.to_owned(),
            depth,
            kind,
            mtime_ns: 0,
        });
    }

    entries
}
