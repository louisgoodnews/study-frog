---
id: STORY-0133
type: story
title: "Console screen in the application"
status: done
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
- [x] A Console screen in the sidebar takes control commands and `script run FILE`, shows each command with its answer, script output and lifecycle events as they happen, keeps a command history (Up/Down) and can be cleared; tested with the egui harness and a screenshot

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`). `console_ui.rs` (state, update), `view/console.rs`, the glue in `app.rs`.

## Progress (2026-11-08)
Built: `console_ui.rs` (state, history, entries from answer and event lines; edited in place), `console_link.rs` (lines go through `control_conn::answer` on a thread into the window's own control queue, `script run|list` to the window's runner, everything comes back through a line sink that wakes the window; subscribed to all lifecycle events from the start), `view/console.rs`, Screen::Console in the sidebar (which now scrolls when the window is too low). Verified: `crates/studyfrog-ui/tests/console.rs` (history and transcript, entry formatting, the screen in the egui harness with typing and Enter, the link against a headless engine with answers, a served `logs`, an error and an `item_created` event, `script run`/`script list` through a real runner with the script's output coming back); a screenshot of the real window after `ctl item create` showed the events and the script output in the Console. Not covered: typing into the Console of a real window by hand (only through the harness); the Console needs the window's control protocol, so with `--no-control` it only says so.

## Dependencies
STORY-0130, STORY-0132.

## Open Questions
None.
