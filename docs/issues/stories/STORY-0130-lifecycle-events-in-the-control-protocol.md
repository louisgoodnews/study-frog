---
id: STORY-0130
type: story
title: "Lifecycle events in the control protocol"
status: done
epic: EPIC-028
parents: [EPIC-028]
estimate: 5
labels: [m12, ui, app]
acceptance_criteria:
  - "The instance derives lifecycle events (vault_opened, vault_closed, item_created, item_saved, rehearsal_finished) from what it applies, in the window and headless, and unit tests show each one is raised once by the matching change"
  - "A control connection can ask for events with `subscribe [kinds...]` and receives them as event lines until it closes; `studyfrog ctl subscribe` streams them; tested over in-memory streams, a real socket and the real binary"
---
## Context
Scripts must react to what happens in StudyFrog (EPIC-028: event-bound triggers). The control protocol (STORY-0124) has no events except followed logs.

## Description
An in-process event bus fed by the instance, and a `subscribe` command served by each connection like `logs --follow`. Tasks: TASK-0243, TASK-0244.

## Acceptance Criteria
- [x] The instance derives lifecycle events (vault_opened, vault_closed, item_created, item_saved, rehearsal_finished) from what it applies, in the window and headless, and unit tests show each one is raised once by the matching change
- [x] A control connection can ask for events with `subscribe [kinds...]` and receives them as event lines until it closes; `studyfrog ctl subscribe` streams them; tested over in-memory streams, a real socket and the real binary

## Design Notes
ADR 0013 (`docs/adr/0013-scripting-out-of-process.md`). `script_events.rs` (pure detection from messages and a state probe), `control_events.rs` (bus, serving). Events are JSON `{kind, at_ms, data}` lines `{"event": "lifecycle", "data": ...}`.

## Progress (2026-11-08)
Built: `script_events.rs` (detection: `item_created` from the create dialog's result, `item_saved` from a save's result, `vault_opened`/`vault_closed` and `rehearsal_finished` from a before/after probe of the state), `control_events.rs` (the bus, `subscribe`, `check_kinds`), applied in the headless engine (every message) and in the window (each frame: worker messages plus a probe over the frame). Verified: `crates/studyfrog-ui/tests/control_events.rs` (each event once on a real vault; a finished run reported once, a stopped one not; a subscribed in-memory connection gets only the kinds it asked for, an unknown kind is an error) and `crates/studyfrog-app/tests/events_binary.rs` (the real binary: `studyfrog ctl subscribe item_created` over the socket receives the item created through standard input). Not covered: item deletion and moves raise no event; the window's detection runs per frame, so a run finished and left in the same frame would be missed (not seen in practice); the window path is not tested in a real window, only the shared detection functions.

## Dependencies
STORY-0124.

## Open Questions
None.
