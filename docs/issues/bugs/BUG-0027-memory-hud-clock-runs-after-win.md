---
id: BUG-0027
type: bug
title: "The Memory HUD keeps counting after the game is won"
status: backlog
estimate: 1
labels: [m14, ui, rehearse]
epic: EPIC-059
parents: ['EPIC-059']
acceptance_criteria:
  - "After the last pair is found, the HUD time stops at the time of the win"
  - "The HUD time and the time in the win message are the same"
---

## Context
Found while auditing STORY-0175 / EPIC-059 (2026-11-22, display :91). After a stopwatch game was won, the win message read "All pairs found in 4:56." while the HUD read "Time 4:59" in the same screen, and "Time 5:29" about 30 s later. Screenshots: `/tmp/claude-0/audit/g1/17-victory.png`, `/tmp/claude-0/audit/g1/18-board-960.png`. The clock message is sent only while the outcome is not set (`crates/studyfrog-ui/src/view/memory/board.rs`), but the HUD keeps reading the live elapsed time.

## Description
Freeze the HUD time at the win time. Reproduction: finish a stopwatch Memory game (Pairs left 0), read the HUD, wait 30 s, read it again. Actual: it keeps rising.

## Acceptance Criteria
- [ ] After the last pair is found, the HUD time stops at the time of the win
- [ ] The HUD time and the time in the win message are the same

## Design Notes
- None.

## Dependencies
- STORY-0175 / EPIC-059

## Open Questions
- None.
