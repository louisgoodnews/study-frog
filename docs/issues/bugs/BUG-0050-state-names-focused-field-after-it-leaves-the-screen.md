---
id: BUG-0050
type: bug
title: "state keeps reporting focused_field 'Query text' after the library query box has left the screen"
status: done
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
- [x] After `query` then `nav home`, `state` reports `focused_field: null`
- [x] While a search field is drawn and has the keyboard, `state` reports its name (keep the current behaviour)

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/control_state.rs` and `crates/studyfrog-ui/src/control_reply/focus.rs`; the focus is read from the egui memory on the UI thread.
- Guardrails: P1 (no inherent `impl` blocks), P2 (<= 250 lines per file).

## Dependencies
STORY-0192 (the focused field in the state snapshot), BUG-0049 (the focus request on Home).

## Open Questions
- Does egui keep the focus id after its widget is not drawn for a frame? If so, the snapshot should compare the id with the widgets drawn in the last frame.

## Fix

- Cause: `state.chrome.focused` was set when a field took the keyboard and never cleared, so `state` kept naming a field that was no longer drawn.
- Fix: each search field records in `view/search_focus.rs` that it has the keyboard in the frame (`track`). When no search field has it at the end of a frame, `end_of_frame` sends `ChromeMsg::SearchFocusLeft`, and `chrome/focus.rs` clears the name. A field that is drawn with the keyboard keeps its name.
- Evidence in the running app: `ctl query status=new`, `ctl search --focus` gave `focused_field: "Query text"`; after `ctl nav home` the state reports `focused_field: null` (the audit binary still reported `"Query text"` on Home). While the library query box has the keyboard, `state` names it.
- Tests: checked with the running app (see BUG-0049); the change is in the egui frame loop, so there is no headless test.
