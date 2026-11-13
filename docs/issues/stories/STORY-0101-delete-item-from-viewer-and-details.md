---
id: STORY-0101
type: story
title: "Delete item from item viewer and editor"
status: done
epic: EPIC-031
parents: [EPIC-031]
estimate: 3
labels: [m11, service, ui]
acceptance_criteria:
  - "Users can delete an open item (note, card, or question) directly from the item editor toolbar with an inline confirmation prompt"
  - "Deleting an item from the editor removes its file from the vault, closes the editor back to the library view, refreshes the item list, and stores an undo snapshot so the deletion can be undone"
---

## Context
Previously, items could only be deleted by selecting their checkboxes in the library table and using the bulk action bar. When viewing or editing a single item on the item page (`Screen::Editor`), there was no action to delete the item directly.

## Description
1. Editor State & Actions (`studyfrog-ui`):
   - Add `confirm_delete: bool` to `Editor` (or manage via `EditAction::AskDelete(bool)` / `EditAction::ConfirmDelete`).
   - Add `"Delete item…"` to the item editor toolbar (`More actions` menu) for items.
   - When triggered, display an inline confirmation bar in the editor toolbar (`"Delete this item?"`, `"Yes, delete"`, `"Cancel"`).
2. Execution & Undo Support:
   - Confirming deletion closes the editor to `Screen::Library` and dispatches `Effect::Library(LibraryEffect::Bulk(vec![uid], BulkEdit::Delete))`, which deletes the item file, updates `VaultInfo`, reloads the current selection, and populates `state.library.undo` so `"Undo last change"` can immediately restore the deleted item.

## Acceptance Criteria
- [x] Users can delete an open item (note, card, or question) directly from the item editor toolbar with an inline confirmation prompt
- [x] Deleting an item from the editor removes its file from the vault, closes the editor back to the library view, refreshes the item list, and stores an undo snapshot so the deletion can be undone

## Design Notes
- Crate touchpoints:
  - `crates/studyfrog-ui/src/edit_form.rs`
  - `crates/studyfrog-ui/src/editor.rs`
  - `crates/studyfrog-ui/src/editor_edit.rs`
  - `crates/studyfrog-ui/src/view/editor.rs`
- Release target: version 0.4.3.

## Dependencies
EPIC-031.

## Open Questions
None.
