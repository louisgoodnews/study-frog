---
id: BUG-0050
type: bug
title: "state keeps reporting focused_field 'Query text' after the library query box has left the screen"
status: backlog
estimate: 1
labels: [m14, tooling, ui]
epic: EPIC-064
parents: ['EPIC-064']
acceptance_criteria:
  - "After the library query box is no longer drawn (navigating to Home, Calendar or Homework), state reports focused_field as null, or as the field that really has the keyboard"
  - "A test or a ctl run shows the focused field matching the screen in front: query -> nav home -> state gives focused_field null"
---

## Context
Found while reviewing STORY-0192 (2026-11-22, `/tmp/claude-0/review-b`). `ctl query status=new` then `ctl search --focus` gives `focused_field: "Query text"`. After `ctl nav home`, `ctl state` still reports `"focused_field": "Query text"` with `library: null` and screen `home`. The same stale name remains on the Calendar and Homework screens. The snapshot is `crates/studyfrog-ui/src/control_state.rs`, and the name comes from the egui memory's focused id.

## Description
The snapshot should name only a field that is drawn on the screen in front and has the keyboard. Here it names a field that no longer exists, so an agent reading `state` would believe a text field has focus when it does not. Either clear the focus when the field is not drawn, or report the focused field only when it was drawn in the last frame.

## Acceptance Criteria
- [ ] After `query` then `nav home`, `state` reports `focused_field: null`
- [ ] While a search field is drawn and has the keyboard, `state` reports its name (keep the current behaviour)

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/control_state.rs` and `crates/studyfrog-ui/src/control_reply/focus.rs`; the focus is read from the egui memory on the UI thread.
- Guardrails: P1 (no inherent `impl` blocks), P2 (<= 250 lines per file).

## Dependencies
STORY-0192 (the focused field in the state snapshot), BUG-0049 (the focus request on Home).

## Open Questions
- Does egui keep the focus id after its widget is not drawn for a frame? If so, the snapshot should compare the id with the widgets drawn in the last frame.
