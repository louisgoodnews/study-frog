# Vault views: Folders and Items

Status: **Specified** for 0.4.0 (STORY-0076, TASK-0202). Implements the owner's decisions of 2026-11-03 and [ADR 0011](../adr/0011-stack-folders-and-references.md).

StudyFrog provides two complementary views of the vault in the sidebar:
1. **Folders**: the physical filesystem hierarchy on disk, modelled directly on Obsidian's file explorer. Used for organizing files, folders, PDFs, pictures, and stack boundaries.
2. **Items**: the logical study catalog (formerly called Library, [library](library.md)). Used for filtering, querying, bulk metadata editing, and rehearsal across stacks.

---

## Sidebar navigation

The sidebar lists both views under the primary study section ([ui-calm](ui-calm.md)):
- **Home**
- **Folders** (icon: folder tree / files)
- **Items** (icon: list / catalog table)
- **Rehearse**

Both views operate on the same files. Choosing an item in either view opens it in the editor. From the Items view or the editor header, the action **Reveal in Folders** switches to the Folders view, expands the directory path, and focuses the file row. Conversely, right-clicking a folder or stack in Folders offers **Show in Items**, filtering the Items view to that folder.

---

## 1. The Folders view

The Folders view gives a calm, responsive, and predictable representation of the vault's directories and files.

### Layout

```
+-------------------------------------------------------------+
| [ + Note ] [ + Folder ] [ Sort v ] [ Expand ] [ Collapse ]  |
+-------------------------------------------------------------+
| v [S] French                                                |
|   |   Verbs                                                 |
|   |     v Regular                                           |
|   |     |   Parler                                          |
|   |     |   Finir                                           |
|   |     Irregular                                           |
|   |-- [S] Exam 2026                                         |
|   |     Practice Test                      [PDF]            |
|   |     Vocabulary Notes                                    |
|   assets                                                    |
|     verb-table.png                         [IMG]            |
|   Inbox Note                                                |
+-------------------------------------------------------------+
```

### Toolbar

A compact toolbar sits pinned at the top of the tree panel:
- **New note** (`+ Note`): creates a new note in the currently selected folder (or the vault root if nothing or the root is selected) and immediately begins inline title editing.
- **New folder** (`+ Folder`): creates a new folder in the currently selected directory. Displays a small creation popover with folder name and a checkbox: `[ ] Mark as stack` (default unchecked; see [TASK-0217](../issues/tasks/TASK-0217-mark-a-folder-as-a-stack-or-as-a-normal-folder.md)).
- **Sort order** (`Sort v`): dropdown menu controlling tree sorting:
  - *File name (A to Z)* (default)
  - *File name (Z to A)*
  - *Modified time (newest first)*
  - *Modified time (oldest first)*
  Folders are always listed before loose files within each folder level.
- **Expand all** (`Expand`): expands all collapsed folders in the tree.
- **Collapse all** (`Collapse`): collapses all open folders back to top-level entries.

### Tree rendering

- **Virtualised rendering**: only visible rows are drawn, maintaining 60 fps scrolling even in vaults with tens of thousands of files.
- **Chevrons**: a downward chevron (`v`) indicates an expanded folder; a rightward chevron (`>`) indicates a collapsed folder. Clicking the chevron toggles expansion without altering selection.
- **Indent guides**: subtle vertical guide lines connect child entries to their parent folder, matching Obsidian's visual structure.
- **Display names**:
  - Markdown notes, cards, and questions display their **file name without the `.md` extension** (e.g. `Parler`, not `Parler.md`).
  - Stack files (`_stack.md`) are **not shown as separate rows** in the tree by default; the enclosing folder itself represents the stack (see below).
  - Non-markdown files display their file name with a small, discreet badge indicating file type:
    - `[PDF]` for `.pdf` documents.
    - `[IMG]` for pictures (`.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`).
    - `[DOC]` or extension badge for other recognized vault attachments.
- **Visible files**:
  - Displays all markdown files, PDFs, and image assets located in the vault.
  - Hidden files and folders (names beginning with `.`, such as `.git`, `.obsidian`, or `.studyfrog`) are hidden.
  - The `assets/` folder is visible like any normal folder, showing embedded images and diagrams.

### Distinguishing stack folders vs normal folders

In accordance with [ADR 0011](../adr/0011-stack-folders-and-references.md):
- A folder is a **stack folder** if and only if it contains a `_stack.md` file.
- A folder without `_stack.md` is a **normal folder**.

Visual distinction in the tree:
- **Stack folder**: renders with a distinct **Stack icon / badge** (`[S]` or layer icon) beside its folder name, using the accent color. It represents both a filesystem folder and an SRS study unit. Double-clicking or selecting "Open Stack" opens the Stack Overview page ([stack-view](stack-view.md)).
- **Normal folder**: renders with a standard neutral folder icon (`📁`). It serves solely for file organization on disk.
- **Stack file visibility**: `_stack.md` is managed automatically by the application and hidden from the regular item rows to avoid accidental edits or confusion. Its metadata (title, description, settings) is edited through the Stack Details dialog or stack header.

Folder context menu options:
- Normal folder: includes **Mark as stack** (writes `_stack.md` with a fresh ULID and the folder name as title).
- Stack folder: includes **Convert to normal folder** (prompts with a confirmation explaining that `_stack.md` will be removed, stack-specific SRS schedules and limits dropped, and contained items reassigned to the parent stack or Inbox; undoable via snapshot).

---

## 2. File and folder operations

All file and directory manipulations run through transactional service plans with atomic execution and undo snapshot recording ([TASK-0203](../issues/tasks/TASK-0203-folder-operations-in-the-service-create-rename-move-delete.md)).

### Primary stack rules on move

1. **Rule of nearest stack**: the primary stack of any item is the nearest stack folder above its file path in the directory tree. If an item sits outside all stack folders, its primary stack is the **Inbox** (`uid: 0`).
2. **Normal folders inherit stack**: placing a normal folder inside a stack folder causes all items within it to belong to that enclosing stack.
3. **Moving within a stack**: moving a file or normal folder within the same stack folder leaves primary stack membership and per-stack SRS schedules intact.
4. **Moving across stack boundaries**:
   - When a move crosses a stack boundary (e.g. from Stack A to Stack B, or into/out of the Inbox), the item's **primary stack changes** to the destination stack.
   - The application displays a confirmation dialog stating:
     > *"Moving N items to 'Stack B'. This will change their primary stack."*
   - **`referenced_by` adjustment**: if the destination stack was previously listed in the item's `referenced_by` front-matter, it is automatically removed (an item never references its own primary stack).
   - **Per-stack memory & schedule**: the old primary stack retains its independent SRS memory (`mem_state`, `sm2_state`, review history) **if and only if** the old stack is added to `referenced_by`. Otherwise, the old stack's SRS memory is dropped, and the confirmation dialog reports the count of schedules affected.

### Link integrity on move and rename

- **Wikilinks by title**: links of the form `[[Item Title]]`, `[[Item Title|Label]]`, or `[[Item Title#Heading]]` resolve by note title and alias, not by file path. Moving files between folders leaves wikilinks fully intact without modification.
- **Renaming a note title**: when a note's file is renamed and its front-matter `title` is updated, the service prompts to refactor wikilinks referencing the old title across the vault.
- **Attachment and PDF links**: relative markdown paths to moved files or pictures (e.g. `![diagram](../assets/diagram.png)` or markdown links to PDFs) are automatically rewritten in all referencing files using canonical vault-relative paths.

### Drag and drop

- Files and folders can be dragged and dropped within the tree to move them into target folders.
- Dropping onto a folder highlights the target folder.
- If the drop targets a different stack, the primary stack reassignment confirmation is triggered before the filesystem plan executes.
- Dropping external files (PDFs, images) from the system desktop or file manager into a folder imports them into that folder and schedules indexing.

### Inline rename and deletion

- **Inline rename**: pressing `F2` or choosing "Rename" turns the label into an inline text box with the current name selected. Pressing `Enter` commits the change; pressing `Escape` cancels.
- **Delete**: pressing `Delete` or choosing "Delete" prompts with a confirmation dialog detailing the number of files and folders to be removed. Deleted files are moved to the system trash or recorded in an undo snapshot.

### External change detection

- Vault file changes triggered outside StudyFrog (e.g. edited in Obsidian, git pull, terminal commands) are picked up by the background file-watcher poll.
- Tree nodes update smoothly without collapsing open folders or stealing user input focus.
- If an active file in the editor is modified externally, the editor notifies the user and reloads cleanly.

---

## 3. Keyboard navigation & shortcuts

The Folders view is fully operable without a mouse:

| Shortcut | Action |
|---|---|
| `Up` / `Down` | Move selection to previous / next visible row in the tree |
| `Right` | Expand collapsed folder; or move to its first child if already open |
| `Left` | Collapse expanded folder; or jump to parent folder if already collapsed |
| `Enter` | Open selected note/card in editor; open PDF in PDF viewer; toggle folder |
| `F2` | Begin inline rename of selected file or folder |
| `Delete` / `Backspace` | Delete selected file or folder (with confirmation prompt) |
| `Ctrl+N` | Create new note in selected folder |
| `Ctrl+Shift+N` | Create new folder in selected folder |
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo previous file or folder operation |
| `Menu` / `Shift+F10` | Open context menu for selected item |

---

## 4. Context menus

### Folder context menu
- **New note** (creates note inside folder)
- **New folder** (creates subfolder)
- **Mark as stack** *(normal folders only)*
- **Convert to normal folder** *(stack folders only)*
- **Show in Items** (switches to Items view filtered to this folder)
- **Rename** (`F2`)
- **Delete** (`Delete`)
- **Copy path** (copies relative vault path)
- **Reveal in system file manager**

### File context menu
- **Open** (`Enter`)
- **Open in new tab**
- **Copy wikilink** (copies `[[Note Title]]` to clipboard)
- **Rename** (`F2`)
- **Move to...** (opens destination folder picker)
- **Delete** (`Delete`)
- **Copy path** (copies relative vault path)
- **Reveal in system file manager**

---

## 5. The Items view

The **Items view** retains the full functionality of the Library ([library](library.md)), serving as the analytical and task-focused catalog of learning content.

### What the Items view shows
- **Filter and search bar**: Standard filter fields (text, type, status, stack, tags, priority, due date) and Query mode syntax (`due <= today and priority = high`).
- **Tabular list**: one row per learning item (note, card, question).
  - Columns: Checkbox, Title (link to open), Type, Status, Priority, Due Date, Familiarity (%), Primary Stack, and "Also in" (`referenced_by` stacks).
  - Column sorting and visibility configuration.
- **Reference display**: for items referenced across multiple stacks, displays the primary stack with an "Also in" chip indicating other referencing stacks. Hovering displays per-stack due dates and familiarity values ([ADR 0011](../adr/0011-stack-folders-and-references.md)).

### Actions in Items view
- **Bulk actions**: select multiple items via checkboxes to set status, priority, difficulty, tags, or primary stack in a single transaction.
- **Manage references**: add or remove stack references (`referenced_by`) across selected items.
- **Start rehearsal**: rehearse items matching the active filter or selection directly.
- **Export**: export filtered or selected items to JSON, TOML, RON, or CSV.
- **Reveal in Folders**: navigates to the Folders view and highlights the file on disk.

---

## Summary of differences

| Dimension | Folders View | Items View |
|---|---|---|
| **Model** | Filesystem tree (Obsidian file explorer) | Structured catalog table (Jira list view) |
| **Rows** | Folders, Markdown files, PDFs, Images | Cards, Notes, Questions (learning items) |
| **Stack representation** | Folder mark (`_stack.md` present) | Column attributes (`stack`, `referenced_by`) |
| **Primary purpose** | Organizing disk hierarchy, moving, renaming | Filtering, learning status, SRS review, bulk metadata |
| **Primary action** | Browse, create files, manage directories | Study, query, filter, bulk update |
