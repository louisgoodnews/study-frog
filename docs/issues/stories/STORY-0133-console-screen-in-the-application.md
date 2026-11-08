---
id: STORY-0133
type: story
title: Console screen in the application
status: backlog
epic: EPIC-028
parents: [EPIC-028]
estimate: 5
labels: [m12, ui]
acceptance_criteria:
  - "A Console screen in the sidebar takes control commands and `script run FILE`, shows each command with its answer, script output and lifecycle events as they happen, keeps a command history (Up/Down) and can be cleared; tested with the egui harness and a screenshot"
---
## Context
EPIC-028 asks for a built-in interactive console.

## Description
The console sends its lines to the window's own control queue, so it speaks exactly the protocol scripts use. Tasks: TASK-0250.

## Acceptance Criteria
- [ ] A Console screen in the sidebar takes control commands and `script run FILE`, shows each command with its answer, script output and lifecycle events as they happen, keeps a command history (Up/Down) and can be cleared; tested with the egui harness and a screenshot

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`). `console_ui.rs` (state, update), `view/console.rs`, the glue in `app.rs`.

## Dependencies
STORY-0130, STORY-0132.

## Open Questions
None.
