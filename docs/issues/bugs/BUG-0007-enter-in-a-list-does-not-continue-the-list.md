---
id: BUG-0007
type: bug
title: Enter in a list does not continue the list
status: backlog
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
- [ ] Enter at the end of a numbered item starts the next number (1., 2., 3.), of a bullet item the next bullet, of a task item the next unchecked box, keeping the indent
- [ ] Enter on an empty item ends the list
- [ ] Backspace at the start of an empty item removes the marker

## Design Notes
The text edits are pure functions in `md_edit.rs` (`apply_cmd`); add an `on_enter` function that reads the current line and returns the new text and cursor, called from the editor's key handling before egui inserts the line break. Tests per list kind, nesting, the empty item and a selection.

## Dependencies
None.

## Open Questions
None.
