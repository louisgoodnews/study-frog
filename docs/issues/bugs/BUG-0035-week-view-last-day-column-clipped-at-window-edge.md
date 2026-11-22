---
id: BUG-0035
type: bug
title: "Week view runs past the window edge, so Sunday's column and its deadline dot are cut off"
status: backlog
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
- [ ] At 1280x800 and at 1600x1000 the whole week fits: all seven day columns and their headers are visible without sideways scrolling
- [ ] A homework deadline on the last day of the week shows its urgency dot in that day's header

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/` (week grid), `view::homework_marks`.
- Guardrails: P6 (≤ 250 lines per file).

## Dependencies
- STORY-0166

## Open Questions
- None.
