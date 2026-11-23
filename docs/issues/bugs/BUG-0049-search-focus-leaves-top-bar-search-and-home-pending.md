---
id: BUG-0049
type: bug
title: "search --focus never takes the keyboard on Home or a stack screen, and the top-bar search field is never focused"
status: done
estimate: 2
labels: [m14, tooling, ui]
epic: EPIC-064
parents: ['EPIC-064']
acceptance_criteria:
  - "search --focus on a screen with a search field of its own (the library, the PDF viewer) gives that field the keyboard and the answer names it (already true for the library query box)"
  - "search --focus on Home, or on a stack screen without a search field, either focuses the top-bar search field or answers that no field on this screen can take the focus, instead of leaving focus pending forever"
  - "the top-bar search field ('Search items') takes the keyboard on search --focus and on Ctrl+F, and the answer names it"
---

## Context
Found while reviewing STORY-0192 (2026-11-22, `/tmp/claude-0/review-b`). `ctl search --focus` on the Home screen and on a stack's Summary tab answers `focus: {focused: null, pending: true}`, and `pending` stays true after several frames and screenshots. The request is only consumed by the library's search fields (`crates/studyfrog-ui/src/view/library_search.rs`) and the PDF find field (`crates/studyfrog-ui/src/view/pdf.rs`). The top-bar field (`crates/studyfrog-ui/src/view/navbar.rs`, hint "Search items") is drawn on every screen but never calls `search_focus::focus_field`.

## Description
Two gaps in the focus request:
1. Screens with no search field of their own leave the request pending indefinitely, so an agent cannot tell that nothing took it.
2. The top-bar search is the search field a user sees on every screen. It should take the keyboard on `search --focus` (and on Ctrl+F, which the story says focuses "the search field of the screen in front"), and it should be named in the answer.

## Acceptance Criteria
- [x] `search --focus` on the library query view names "Query text" and `state` reports it as `focused_field` (true today; keep it)
- [x] On Home and on a stack Summary, `search --focus` either focuses the top-bar field or answers that the screen has no field to take the focus, rather than `pending: true` forever
- [x] The top-bar "Search items" field takes the keyboard on `search --focus`, and the answer and `state` name it

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/navbar.rs` (call `focus_field` and `taken` on the top-bar field), `crates/studyfrog-ui/src/chrome/focus.rs` (clear the request when no field is drawn).
- Guardrails: P1 (no inherent `impl` blocks), P2 (<= 250 lines per file).

## Dependencies
STORY-0192 (search focus in the control protocol), STORY-0188 (Ctrl+F and `/`).

## Open Questions
- Should the top-bar field be the target on screens that have no search field of their own, or should the answer report "no field" there?

## Fix

- Cause: the request was consumed only by the library and PDF fields. The top-bar field never took it, so on screens without a field of their own the request stayed pending.
- Fix: `view/search_focus.rs::end_of_frame` (called from `model::frame` after the views are drawn) gives the top-bar field "Search items" the keyboard when no page field took the request or the shortcut. The top bar requests the focus in the frame after it is drawn, because egui drops a focus requested after its widget was drawn. The library query box and the PDF find field keep taking the request first.
- Evidence in the running app (window at 0,0): before the fix, `search --focus` on Home answered `{"focused": null, "pending": true}` (audit binary). After the fix: Home and a stack's screen answer `{"focused": "Search items", "pending": false}`, typed text lands in the top-bar field, Ctrl+F on Home focuses it, and `query status=new` then `search --focus` still answers `Query text`.
- Tests: the focus is egui-window behaviour; there is no headless test. Checked with `ctl search --focus`, `ctl state`, xdotool typing and screenshots. `crates/studyfrog-ui/tests/control_focus_session.rs` and `chrome_views.rs` pass.
