---
id: BUG-0025
type: bug
title: "In Memory time attack the HUD shows the time used, not the time left, and reads past the limit"
status: done
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

**Fix (2026-11-22).** The HUD shows the time left in a time attack (`studyfrog-rehearsal/src/memory_summary.rs` `hud_clock_ms`), and the danger colours still follow the time left. Test: `studyfrog-rehearsal/tests/memory_summary.rs` `the_hud_clock_counts_down_in_a_time_attack_and_stops_at_zero`. Running app: a 60 s game reads "Time 0:52" about 8 s in (`/tmp/claude-0/fix1/0025-hud-after-7s.png`) and "Time 0:00" at the timeout (`/tmp/claude-0/fix1/0026-0028-timeout-after-960.png`). Reading of the second criterion: at the timeout the HUD reads 0:00, the time left, which the first criterion requires; it never shows more than the limit.

## Description
Show the remaining time in time-attack mode (the stopwatch keeps the elapsed time). Keep the danger colours tied to the time left.

## Acceptance Criteria
- [x] In a 60, 90 or 120 s time attack the HUD shows the time left, counting down to 0:00
- [x] The HUD never shows more than the chosen limit in a time attack; at the timeout it reads the limit

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
