---
id: BUG-0049
type: bug
title: "search --focus never takes the keyboard on Home or a stack screen, and the top-bar search field is never focused"
status: backlog
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
- [ ] `search --focus` on the library query view names "Query text" and `state` reports it as `focused_field` (true today; keep it)
- [ ] On Home and on a stack Summary, `search --focus` either focuses the top-bar field or answers that the screen has no field to take the focus, rather than `pending: true` forever
- [ ] The top-bar "Search items" field takes the keyboard on `search --focus`, and the answer and `state` name it

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/navbar.rs` (call `focus_field` and `taken` on the top-bar field), `crates/studyfrog-ui/src/chrome/focus.rs` (clear the request when no field is drawn).
- Guardrails: P1 (no inherent `impl` blocks), P2 (<= 250 lines per file).

## Dependencies
STORY-0192 (search focus in the control protocol), STORY-0188 (Ctrl+F and `/`).

## Open Questions
- Should the top-bar field be the target on screens that have no search field of their own, or should the answer report "no field" there?
