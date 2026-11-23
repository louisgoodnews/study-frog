---
id: BUG-0035
type: bug
title: "Week view runs past the window edge, so Sunday's column and its deadline dot are cut off"
status: done
estimate: 2
labels: [m14, calendar, ui]
acceptance_criteria:
  - "At 1280x800 and at 1600x1000 the whole week fits: all seven day columns and their headers are visible without sideways scrolling"
  - "A homework deadline on the last day of the week shows its urgency dot in that day's header"
---

## Context
Found while auditing STORY-0166 in the running app (2026-11-22). In the week view the day columns are wider than the window: at 1280 px the Sun 22 column starts at x=1168 and is cut off at the window edge, and at 1600 px it still runs under the vertical scroll bar. Sunday's header has a homework deadline ("Statistik Blatt 3", overdue, due 18:00 on 22 Nov), but its urgency dot sits at the right edge of the column and is not visible at either size. Saturday's dot is visible.

Screenshots: `/tmp/claude-0/audit/g5/0166-week.png` (1280x800), `/tmp/claude-0/audit/g5/0166-week1600.png` (1600x1000).

## Description
Fit the seven day columns and the time-label gutter into the available width, or make the grid scroll sideways as a whole, so the last column and its header dot are reachable. Check the header dots of every day with a deadline.

## Acceptance Criteria
- [x] At 1280x800 and at 1600x1000 the whole week fits: all seven day columns and their headers are visible without sideways scrolling
- [x] A homework deadline on the last day of the week shows its urgency dot in that day's header

## Design Notes
- Fixed on 2026-11-23. Cause: `view/calendar_week.rs` sized the seven day columns from the width without the item gaps of the rows (eight per row), so the last column ran past the right edge: at 1280 Sunday's column ended near x=1313 in a 1280-wide window, and its header dot was off-screen. Fix: the column width subtracts the gaps and the scroll bar room, and the day headers carry their label. Before: `/tmp/claude-0/layout/before-week-1280.png`. After: `/tmp/claude-0/layout/after4-week-1280.png` (Sunday's column and its deadline dot inside the window). Test: `studyfrog-ui/tests/window_fit_layout.rs::the_whole_week_fits_the_window_at_1280_and_1600` (checks 1280 by 800 and 1600 by 900; the app's virtual display is 1280 by 800, so 1600 was not seen in the app; the 1600 by 1000 size in the acceptance was not run).
- Touchpoints: `crates/studyfrog-ui/src/view/` (week grid), `view::homework_marks`.
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0166

## Open Questions
- None.
