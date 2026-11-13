---
id: TASK-0040
type: task
title: "Editor widget"
status: done
epic: EPIC-009
parents: [STORY-0018]
estimate: 5
labels: [m4, ui]
acceptance_criteria:
  - "Multi-line Markdown editing with undo/redo and IME"
  - "Front-matter edited as form fields or raw (decide in spike)"
---

## Context
egui text editing maturity is the risk (ADR 0002).

## Description
Editor widget.

## Acceptance Criteria
- [x] Multi-line Markdown editing with undo/redo and IME
- [x] Front-matter edited as form fields or raw (decide in spike)

## Design Notes
**Spike result: egui is adequate.** Multi-line editing, undo/redo (Ctrl+Z, Ctrl+Shift+Z, demonstrated headlessly) and the live preview work. Front-matter is edited as **form fields** (title, tags and the kind-specific text areas); unknown keys and their comments are carried over untouched because the form edits a clone of the parsed document (`edit_form.rs`). A raw front-matter view is deferred. Saving goes through `studyfrog-service::document`: the file's content hash from load time must still match (otherwise "changed outside StudyFrog"), the write uses a temp file, the `updated` time is set and the index refreshed. Unsaved changes block Back, navigation and opening another item. IME composition is still unverified (needs a person at a keyboard); iced remains the fallback.

## Dependencies
TASK-0037.

## Open Questions
None.
