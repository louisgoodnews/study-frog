---
id: BUG-0056
type: bug
title: "Note editor details column clips priority and difficulty chips at 1280x800"
status: done
epic: EPIC-036
parents: ['EPIC-036']
estimate: 1
labels: [m14, ui]
acceptance_criteria:
  - "At 1280x800 and 960x640 every priority and difficulty chip in the note editor's details column is visible in full (no label cut at the column edge)"
  - "Every control of the details column stays reachable: the chips wrap onto a new line rather than being removed or hidden"
---

## Context
Seen by the editor agent in commit 878aa9a (screenshot `/tmp/claude-0/sidebar/after-1280.png`). In the Properties tab of the note editor's details column at 1280x800, the "difficult" difficulty chip and the "medium" priority chip are cut at the right edge of the column, and the Tags field runs past it. The column is a both-direction scroll area, so its content is laid out with an unbounded width and the chip rows never wrap at the column width.

## Description
Make the details column wrap its rows at the column's own width, so that the priority and difficulty chips fit at 1280x800 and at 960x640.

## Acceptance Criteria
- [x] At 1280x800 and 960x640 every priority and difficulty chip in the note editor's details column is visible in full (no label cut at the column edge)
- [x] Every control of the details column stays reachable: the chips wrap onto a new line rather than being removed or hidden

## Design Notes
- Cause: `crates/studyfrog-ui/src/view/editor.rs` (`page`) puts the details in `ScrollArea::both()`, whose children get an unbounded available width, so `horizontal_wrapped` in `crates/studyfrog-ui/src/view/item_details/tab_properties.rs` (`priority_row`, `difficulty_row`) never wraps.
- Fix: bound the content to the column width (`ui.set_max_width(side)`) inside the scroll area. Both directions stay, so wide rows still scroll rather than leave the window.
- Guardrails: P6 (files under 250 lines), no literal font sizes, text buttons keep `.on_hover_text`.

## Dependencies
- EPIC-036

## Open Questions
- None.

## Fix note (2026-11-23): fixed, verified in the running app

Cause: the column's scroll area had no width bound, so the card and its chip rows were laid out wider than the column, past the window's edge. The tab row (Properties, Relations, Learning, Comments) did not wrap, and it widened the column further.

Fix: `crates/studyfrog-ui/src/view/editor.rs` (`page`) bounds the column to its width and scrolls vertically only; `crates/studyfrog-ui/src/view/item_details/mod.rs` (`tab_bar`) wraps the tabs. The chips then wrap inside the column, and the lower controls (colour swatches, status, dates) are reached by scrolling.

Before (commit 878aa9a): at 1280x800 "medium" (priority) and "difficult" are cut at the column edge (`/tmp/claude-0/sidebar/after-1280.png`); at 960x640 "medi" and "diffi" are cut (`/tmp/claude-0/sidebar/before-960.png`).

After: at 1280x800 every priority and difficulty chip is whole, and Comments sits on a second tab row (`/tmp/claude-0/b55/d6-test.png`). At 960x640 the chips are inside the column and the rest scrolls down (`/tmp/claude-0/b55/d7-editor-960-after.png`, and `/tmp/claude-0/b55/d8-960-scrolled.png` with the colour swatches and status reached).

No unit test: the layout was checked in the running app only.
