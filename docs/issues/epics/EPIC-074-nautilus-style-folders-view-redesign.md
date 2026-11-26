---
id: EPIC-074
type: epic
title: "Folders view redesign: Nautilus/Thunar-style file manager with view switcher, icon grid, detailed list, and sorting"
status: done
parents: []
labels: [m14, ui, store]
acceptance_criteria:
  - 'The Folders view provides a Nautilus/Thunar-style file manager header with Back/Forward/Up navigation, clickable breadcrumb path bar, quick create (+ Folder / + Note), inline filter/search, and a view switcher toggling between Icon Grid, Detailed List, and Tree views'
  - 'Icon Grid view renders a responsive grid of recognizable vector icons for folders, stack folders, notes, cards, questions, PDFs, and media files with wrapped labels, selection highlights, double-click/Enter navigation, and drag-and-drop move support'
  - 'Detailed List view renders a full-width multi-column table with clickable sortable column headers (Name, Kind/Type, Size or Item Count, Modified date) and sort-direction indicators, plus star/bookmark toggling'
  - 'Sorting controls support sorting by Name, Modified date, Kind/Type, and Size/Item Count in ascending or descending order, with an optional Folders-first toggle, persisted across sessions and shared across views'
  - 'A compact status bar/pill reports the current selection (name, size or child item count, multi-selection count) and all existing folder actions (create, rename, move, delete, mark as stack, open item/PDF, context menu, keyboard navigation) work in every view mode'
---

## Context
The current Folders screen (`crates/studyfrog-ui/src/view/folders/`) displays only a narrow indented tree list inside a large empty container with minimal toolbar controls (`+ Note`, `+ Folder`, `File name (A to Z)`, `Expand`, `Collapse`). On vaults with nested stacks, notes, cards, and PDFs, it feels cramped, under-utilizes available screen space, and lacks the visual clarity and navigation ergonomics of desktop file managers like GNOME Files (Nautilus) or Thunar.

## Description
Redesign the Folders screen into a modern, desktop-grade Nautilus/Thunar-style vault file manager:
1. **File Manager Navigation & Header Bar**:
   - Back, Forward, and Up-to-parent navigation buttons scoped to the Folders view's directory history.
   - Clickable **Breadcrumb Path Bar** showing the current vault folder path (e.g. `Vault > psychologische-diagnostik-und-evaluation > ...`) so users can jump to any ancestor folder in one click.
   - Quick actions (`+ Folder`, `+ Note`, inline search/filter within current folder) and a **View Switcher** button group + view options popover (Icon Grid, Detailed List, optional Compact/Tree mode, icon zoom/density, show hidden files, folders-first).
2. **Icon Grid View (Nautilus-style)**:
   - Responsive multi-column grid filling the canvas width with clean vector-drawn icons distinguishing regular folders, stack folders (`[S]` badge/tint), Markdown notes, flashcards, questions, PDFs, and images.
   - Centered, cleanly wrapped/ellipsized filenames, hover and selection card highlights, single-click select (with multi-select via Ctrl/Shift), double-click or Enter to open a folder/item/PDF, and drag-and-drop onto folder tiles.
3. **Detailed List View (Nautilus/Thunar-style)**:
   - Full-width tabular list with clickable, sortable column headers: **Name** (with icon + sort chevron), **Type / Kind**, **Size / Items** (e.g. `3 items` for folders, file size for files), and **Modified** (relative human-readable date/time such as `Today, 10:10`, `Yesterday`, `Last month`), plus optional quick **Star / Bookmark** action.
   - Preserves right-click context menus, F2 inline rename, and drag-and-drop onto folder rows.
4. **Rich Sorting & Filtering**:
   - Sort by **Name**, **Modified**, **Type / Kind**, or **Size / Item Count**, ascending or descending, from both the view-options menu and clickable Detailed List column headers.
   - Option to keep **Folders before files** and remember the user's preferred view mode and sort order in app settings.
5. **Quick Places & Selection Status Pill**:
   - Optional compact Places / Quick-Access strip or pane (Vault Root, Recent, Starred/Bookmarked folders, Stacks, PDFs) for fast jumping without crowding the main sidebar.
   - Bottom-right Nautilus-style selection status pill (e.g. `"1.Organisation und Einführung.pdf" selected (2.4 MB)` or `3 items selected`).

## Acceptance Criteria
- [x] The Folders view provides a Nautilus/Thunar-style file manager header with Back/Forward/Up navigation, clickable breadcrumb path bar, quick create (`+ Folder` / `+ Note`), inline filter/search, and a view switcher toggling between Icon Grid, Detailed List, and Tree views
- [x] Icon Grid view renders a responsive grid of recognizable vector icons for folders, stack folders, notes, cards, questions, PDFs, and media files with wrapped labels, selection highlights, double-click/Enter navigation, and drag-and-drop move support
- [x] Detailed List view renders a full-width multi-column table with clickable sortable column headers (Name, Kind/Type, Size or Item Count, Modified date) and sort-direction indicators, plus star/bookmark toggling
- [x] Sorting controls support sorting by Name, Modified date, Kind/Type, and Size/Item Count in ascending or descending order, with an optional Folders-first toggle, persisted across sessions and shared across views
- [x] A compact status bar/pill reports the current selection (name, size or child item count, multi-selection count) and all existing folder actions (create, rename, move, delete, mark as stack, open item/PDF, context menu, keyboard navigation) work in every view mode

## Design Notes
- Existing folder state, navigation, worker execution, and views live in:
  - `crates/studyfrog-ui/src/folders_types.rs`
  - `crates/studyfrog-ui/src/folders_tree.rs`
  - `crates/studyfrog-ui/src/folders_nav.rs`
  - `crates/studyfrog-ui/src/folders_ui.rs`
  - `crates/studyfrog-ui/src/exec_folders.rs`
  - `crates/studyfrog-ui/src/view/folders/`
- Reuse existing `studyfrog-service` folder operations (create, rename, move, delete, mark as stack, undo snapshots) and extend folder metadata entries with file size, child count, and modified timestamp where not already populated.
- Draw crisp vector folder/file icons with `egui::Painter` using `palette_of(ui)` tokens (no raw RGB literals, theme-aware across light/dark themes) and adhere strictly to Invariant P1 (pure free functions), Invariant P6 ($\le 250$ LOC per file), and accessible widget naming for `ui tree` / `studyfrog ctl`.

## Dependencies
- Existing Folders view and service operations (`TASK-0203`, `TASK-0204`, `STORY-0139`, done)

## Open Questions
- Default view mode is **Icon Grid** (`FoldersViewMode::Grid`), with **Detailed List** and **Tree** one click away in the View Switcher.

## Implementation & Verification (2026-11-26)
- Pure browse/sort/filter/navigation state and helpers in `crates/studyfrog-ui/src/folders_browse.rs` (`FoldersViewMode`, `SortColumn`, `BrowseMsg`, `apply_browse`, `visible_dir_entries`, `breadcrumbs`, `compare_with_options`).
- Extended `FolderEntry` with `size_bytes` populated during vault scan (`exec_folders.rs`) and `SortOrder` with `KindAsc/Desc` and `SizeDesc/Asc`.
- Built Nautilus/Thunar-style UI modules in `crates/studyfrog-ui/src/view/folders/`: `icons.rs` (vector icons using `palette_of(ui)`), `toolbar.rs` (Back/Forward/Up, clickable breadcrumbs, inline filter, View Switcher, Sort combo, Folders-first, Starred filter), `grid.rs` (responsive Icon Grid), `list.rs` (sortable multi-column Detailed List with star toggles), and `status.rs` (selection status pill).
- Verified with `cargo fmt --all --check`, `cargo clippy -p studyfrog-ui --all-targets -- -D warnings`, `scripts/linecount.sh 250`, `scripts/check-impls.sh`, `scripts/check-deps.sh`, and headless UI suites (`folders_nautilus_ui`, `folders_view_ui`, `folders_dialogs_ui`, `visual_rhythm` — all passing).
