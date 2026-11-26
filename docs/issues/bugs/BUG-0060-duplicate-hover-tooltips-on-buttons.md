---
id: BUG-0060
type: bug
title: "Duplicate hover tooltips rendered on certain buttons (e.g. Rehearse 'Show answer (Space)')"
status: backlog
estimate: 1
labels: [m14, ui]
acceptance_criteria:
  - 'Hovering "Show answer (Space)" (and "Show answer without answering (Space)") in Rehearse displays a single tooltip instead of two stacked identical tooltips'
  - 'Audit all views in crates/studyfrog-ui/src/view/ for chained or duplicate .on_hover_text(...) calls on the same widget response and remove any duplicates'
  - 'Add a regression check (or extend text_button_tooltips) to catch duplicate .on_hover_text(...) calls on a single button'
---

## Context
Reported by the user with a screenshot of the Rehearse screen: hovering the `"Show answer (Space)"` button displays two identical stacked tooltip bubbles:
> `Shows the answer.`
> `Shortcut: Space`
>
> `Shows the answer.`
> `Shortcut: Space`

Direct cause in `crates/studyfrog-ui/src/view/run_card/parts.rs`: `.on_hover_text(crate::view::tips::show_answer())` is attached inside both branches of an `if/else` expression (lines 76 & 78) and then chained a second time on the resulting `Response` (line 81). Other buttons across `crates/studyfrog-ui/src/view/` should be audited for the same pattern.

## Description
1. Remove the redundant `.on_hover_text(...)` call in `crates/studyfrog-ui/src/view/run_card/parts.rs`.
2. Audit `crates/studyfrog-ui/src/view/` for any other widgets that chain `.on_hover_text(...)` twice (either directly, across `if/else` branches, or via helper functions that already attach a hover tip) and deduplicate them.
3. Add a regression test ensuring no button attaches duplicate hover tooltips.

## Acceptance Criteria
- [ ] Hovering `"Show answer (Space)"` (and `"Show answer without answering (Space)"`) in Rehearse displays a single tooltip instead of two stacked identical tooltips
- [ ] Audit all views in `crates/studyfrog-ui/src/view/` for chained or duplicate `.on_hover_text(...)` calls on the same widget response and remove any duplicates
- [ ] Add a regression check (or extend `text_button_tooltips`) to catch duplicate `.on_hover_text(...)` calls on a single button

## Design Notes
- Check `crates/studyfrog-ui/src/view/run_card/parts.rs:70-85` and scan `crates/studyfrog-ui/src/view/**/*.rs` for duplicate `on_hover_text` / `on_disabled_hover_text` chains.
- Preserve `text_button_tooltips` test compliance.

## Dependencies
None.

## Open Questions
None.
