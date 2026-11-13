---
id: BUG-0007
type: bug
title: "Enter in a list does not continue the list"
status: done
epic: EPIC-026
parents: [STORY-0074]
estimate: 3
labels: [m11, ui, md]
acceptance_criteria:
  - "Enter at the end of a numbered item starts the next number (1., 2., 3.), of a bullet item the next bullet, of a task item the next unchecked box, keeping the indent"
  - "Enter on an empty item ends the list"
  - "Backspace at the start of an empty item removes the marker"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
The Numbered button puts `1. ` at the start of the selected lines, and pressing Enter at the end of that line gives a plain new line: no `2.` follows, and the same for bullets.

## Acceptance Criteria
- [x] Enter at the end of a numbered item starts the next number (1., 2., 3.), of a bullet item the next bullet, of a task item the next unchecked box, keeping the indent
- [x] Enter on an empty item ends the list
- [x] Backspace at the start of an empty item removes the marker

## Design Notes
The text edits are pure functions in `md_edit.rs` (`apply_cmd`); add an `on_enter` function that reads the current line and returns the new text and cursor, called from the editor's key handling before egui inserts the line break. Tests per list kind, nesting, the empty item and a selection.

Built on 2026-11-03: `md_lists.rs` (pure `on_enter` and `on_backspace`) is called from the editor's key handling (`view/md_editor/keys.rs`) before egui inserts the line break, and only when the edit is a list edit, so other lines keep the editor's own Enter and Backspace. Enter in the middle of an item splits it. Backspace at the end of an item with no text removes the marker. Tests in `tests/md_lists.rs` cover the pure edits and the real editor. Quotes are not continued (not asked for).

## Dependencies
None.

## Open Questions
None.
