---
id: TASK-0295
type: task
title: "Saved classroom sessions browser, resume flow, headless tests, and control-protocol hooks"
status: done
epic: EPIC-068
parents: ['STORY-0211']
estimate: 2
labels: [m14, ui, tooling]
acceptance_criteria:
  - 'Users can browse persisted classroom sessions (title, topic, date, turn/task counts), resume any session where it left off, export its transcript to Markdown, or delete it with confirmation'
  - 'Control protocol supports navigating to the Classroom screen and inspecting active/saved session state'
  - 'Headless egui_kittest tests cover session creation, chat display, assignment turn-in, grade rendering, and resuming a persisted session'
---

## Context
Part of `STORY-0211`. Completes the local persistence loop in the UI and ensures the Classroom screen is testable both headlessly and via `studyfrog ctl`.

## Description
Add session-history management, control-protocol integration, and headless UI tests:
- Saved sessions list with resume, Markdown export, and confirmed delete actions.
- `nav classroom` and session state reporting in `studyfrog-ui`'s control engine.
- Headless `egui_kittest` suite verifying layout, accessible widget names, turn-in locking, and session reload.

## Acceptance Criteria
- [x] Users can browse persisted classroom sessions (title, topic, date, turn/task counts), resume any session where it left off, export its transcript to Markdown, or delete it with confirmation
- [x] Control protocol supports navigating to the Classroom screen and inspecting active/saved session state
- [x] Headless `egui_kittest` tests cover session creation, chat display, assignment turn-in, grade rendering, and resuming a persisted session

## Design Notes
- Follow the existing `tests/ollama_panel_ui.rs` and `tests/ollama_control.rs` patterns for headless UI and control-protocol tests.

## Dependencies
- `STORY-0211`, `TASK-0288`, `TASK-0293`, `TASK-0294`

## Open Questions
None.
