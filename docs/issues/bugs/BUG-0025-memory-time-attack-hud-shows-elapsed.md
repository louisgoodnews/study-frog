---
id: BUG-0025
type: bug
title: "In Memory time attack the HUD shows the time used, not the time left, and reads past the limit"
status: backlog
estimate: 2
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "In a 60, 90 or 120 s time attack the HUD shows the time left, counting down to 0:00"
  - "The HUD never shows more than the chosen limit in a time attack; at the timeout it reads the limit"
---

## Context
Found while auditing STORY-0175 / EPIC-059 (2026-11-22, display :91). In time-attack mode the HUD prints the elapsed time (`s.elapsed_ms`) and colours it by the time left. Screenshots: `/tmp/claude-0/audit/g1/13-match.png` (HUD "Time 0:53" in red, so 7 s were left, but the number is the time used), `/tmp/claude-0/audit/g1/14-timeout.png` (HUD "Time 1:04" after a 60 s limit). Code: `crates/studyfrog-ui/src/view/memory/board.rs` `fn hud`.

## Description
Show the remaining time in time-attack mode (the stopwatch keeps the elapsed time). Keep the danger colours tied to the time left.

## Acceptance Criteria
- [ ] In a 60, 90 or 120 s time attack the HUD shows the time left, counting down to 0:00
- [ ] The HUD never shows more than the chosen limit in a time attack; at the timeout it reads the limit

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
