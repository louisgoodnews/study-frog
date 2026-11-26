//! Pure browsing, view-mode, sorting, filtering and formatting logic for the Folders view (EPIC-074).

use std::cmp::Ordering;

use crate::folders_types::{EntryKind, FolderEntry, FoldersState, SortOrder};

/// Presentation mode of the Folders file manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FoldersViewMode {
    /// Nautilus-style responsive icon grid.
    #[default]
    Grid,
    /// Nautilus/Thunar-style detailed sortable table.
    List,
    /// Classic indented folder tree.
    Tree,
}

/// Sortable column in the Detailed List view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    /// File or folder name.
    Name,
    /// Entry kind/type.
    Kind,
    /// File size or folder child count.
    Size,
    /// Modification timestamp.
    Modified,
}

/// Browsing, navigation, view-switcher and filtering messages for the Folders view.
#[derive(Debug, Clone, PartialEq)]
pub enum BrowseMsg {
    /// Switch between Grid, List, and Tree views.
    SetViewMode(FoldersViewMode),
    /// Enter a directory (`""` = vault root), recording history.
    OpenDir(String),
    /// Navigate back in folder history.
    NavBack,
    /// Navigate forward in folder history.
    NavForward,
    /// Navigate to parent folder.
    NavUp,
    /// Update inline filter text.
    SetFilter(String),
    /// Toggle folders-first ordering.
    ToggleFoldersFirst,
    /// Toggle star/bookmark on an entry path.
    ToggleStar(String),
    /// Toggle showing only starred entries.
    ToggleStarredOnly,
    /// Click a Detailed List column header to sort or reverse direction.
    ToggleSortColumn(SortColumn),
}

/// Apply a browse/navigation message to `FoldersState`.
pub fn apply_browse(f: &mut FoldersState, msg: BrowseMsg) {
    match msg {
        BrowseMsg::SetViewMode(mode) => f.view_mode = mode,
        BrowseMsg::OpenDir(dir) => {
            if f.current_dir != dir {
                f.nav_back.push(std::mem::take(&mut f.current_dir));
                f.nav_forward.clear();
                f.current_dir = dir;
                f.selected = None;
                f.show_starred_only = false;
            }
        }
        BrowseMsg::NavBack => {
            if let Some(prev) = f.nav_back.pop() {
                f.nav_forward.push(std::mem::take(&mut f.current_dir));
                f.current_dir = prev;
                f.selected = None;
                f.show_starred_only = false;
            }
        }
        BrowseMsg::NavForward => {
            if let Some(next) = f.nav_forward.pop() {
                f.nav_back.push(std::mem::take(&mut f.current_dir));
                f.current_dir = next;
                f.selected = None;
                f.show_starred_only = false;
            }
        }
        BrowseMsg::NavUp => {
            if !f.current_dir.is_empty() {
                let parent = f
                    .current_dir
                    .rsplit_once('/')
                    .map_or(String::new(), |(p, _)| p.to_owned());
                f.nav_back.push(std::mem::take(&mut f.current_dir));
                f.nav_forward.clear();
                f.current_dir = parent;
                f.selected = None;
                f.show_starred_only = false;
            }
        }
        BrowseMsg::SetFilter(q) => f.filter = q,
        BrowseMsg::ToggleFoldersFirst => f.folders_mixed = !f.folders_mixed,
        BrowseMsg::ToggleStar(path) => {
            if !f.starred.remove(&path) {
                f.starred.insert(path);
            }
        }
        BrowseMsg::ToggleStarredOnly => f.show_starred_only = !f.show_starred_only,
        BrowseMsg::ToggleSortColumn(col) => f.sort = next_sort_for_column(f.sort, col),
    }
}

/// Toggle sort order when a column header is clicked.
pub fn next_sort_for_column(cur: SortOrder, col: SortColumn) -> SortOrder {
    match col {
        SortColumn::Name => match cur {
            SortOrder::NameAsc => SortOrder::NameDesc,
            _ => SortOrder::NameAsc,
        },
        SortColumn::Kind => match cur {
            SortOrder::KindAsc => SortOrder::KindDesc,
            _ => SortOrder::KindAsc,
        },
        SortColumn::Size => match cur {
            SortOrder::SizeDesc => SortOrder::SizeAsc,
            _ => SortOrder::SizeDesc,
        },
        SortColumn::Modified => match cur {
            SortOrder::MtimeDesc => SortOrder::MtimeAsc,
            _ => SortOrder::MtimeDesc,
        },
    }
}

/// Direct child count for a folder path.
pub fn child_count(entries: &[FolderEntry], folder_path: &str) -> usize {
    entries.iter().filter(|e| e.parent == folder_path).count()
}

/// Display name of an entry (`.md` stripped for markdown notes).
pub fn display_name(entry: &FolderEntry) -> String {
    if entry.kind == EntryKind::Markdown {
        entry
            .name
            .strip_suffix(".md")
            .unwrap_or(&entry.name)
            .to_owned()
    } else {
        entry.name.clone()
    }
}

/// Human-readable kind label for the Detailed List view.
pub fn kind_label(entry: &FolderEntry) -> String {
    if entry.is_folder {
        return if entry.is_stack { "Stack" } else { "Folder" }.to_owned();
    }
    match &entry.kind {
        EntryKind::Folder => "Folder".to_owned(),
        EntryKind::Markdown => "Markdown".to_owned(),
        EntryKind::Pdf => "PDF".to_owned(),
        EntryKind::Image => "Image".to_owned(),
        EntryKind::Other(ext) => ext.clone(),
    }
}

/// Compare two entries using sort order, folders-first flag, and folder child counts.
pub fn compare_with_options(
    a: &FolderEntry,
    b: &FolderEntry,
    sort: SortOrder,
    folders_first: bool,
    all: &[FolderEntry],
) -> Ordering {
    if folders_first && a.is_folder != b.is_folder {
        return if a.is_folder {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    let primary = match sort {
        SortOrder::NameAsc => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        SortOrder::NameDesc => b.name.to_lowercase().cmp(&a.name.to_lowercase()),
        SortOrder::MtimeDesc => b.mtime_ns.cmp(&a.mtime_ns),
        SortOrder::MtimeAsc => a.mtime_ns.cmp(&b.mtime_ns),
        SortOrder::KindAsc => kind_label(a).cmp(&kind_label(b)),
        SortOrder::KindDesc => kind_label(b).cmp(&kind_label(a)),
        SortOrder::SizeDesc => entry_metric(b, all).cmp(&entry_metric(a, all)),
        SortOrder::SizeAsc => entry_metric(a, all).cmp(&entry_metric(b, all)),
    };
    primary.then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
}

fn entry_metric(e: &FolderEntry, all: &[FolderEntry]) -> u64 {
    if e.is_folder {
        child_count(all, &e.rel_path) as u64
    } else {
        e.size_bytes
    }
}

/// Visible entries for the active directory (or starred filter), sorted and filtered.
pub fn visible_dir_entries<'a>(
    entries: &'a [FolderEntry],
    folders: &FoldersState,
) -> Vec<&'a FolderEntry> {
    let q = folders.filter.trim().to_lowercase();
    let mut list: Vec<&FolderEntry> = entries
        .iter()
        .filter(|e| {
            let in_scope = if folders.show_starred_only {
                folders.starred.contains(&e.rel_path)
            } else {
                e.parent == folders.current_dir
            };
            in_scope && (q.is_empty() || e.name.to_lowercase().contains(&q))
        })
        .collect();
    list.sort_by(|a, b| compare_with_options(a, b, folders.sort, !folders.folders_mixed, entries));
    list
}

/// Breadcrumb segments `(label, rel_path)` from vault root to `current_dir`.
pub fn breadcrumbs(current_dir: &str) -> Vec<(String, String)> {
    let mut out = vec![("Vault".to_owned(), String::new())];
    if current_dir.is_empty() {
        return out;
    }
    let mut acc = String::new();
    for part in current_dir.split('/') {
        if !acc.is_empty() {
            acc.push('/');
        }
        acc.push_str(part);
        out.push((part.to_owned(), acc.clone()));
    }
    out
}
