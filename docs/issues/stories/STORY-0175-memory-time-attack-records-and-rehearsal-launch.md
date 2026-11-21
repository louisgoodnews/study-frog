---
id: STORY-0175
type: story
title: "Time-Attack Countdown Mode, Session Records, and Rehearsal Launch Entry Points"
status: review
epic: EPIC-059
parents: ['EPIC-059']
estimate: 2
labels: [m14, rehearse, ui]
acceptance_criteria:
  - 'Time-attack mode with countdown options (e.g. 60s, 90s, 120s) and timeout defeat condition'
  - 'Victory summary modal detailing elapsed time, move accuracy, speed score, and personal best records'
  - 'Memory Mode launch action integrated into the Rehearse setup screen and Stack detail view'
---

## Context
To make Memory Mode an exciting game against time, learners need structured time-attack modes, completion scoring, personal best records, and convenient launch triggers directly from their stacks.

## Description
Complete the user journey for Memory Mode:
- Add mode options in setup: Stopwatch (open-ended fastest clear) vs. Time Attack (fixed countdown with danger timer coloration as seconds run out).
- Implement victory & defeat summaries:
  - Victory: Display final time, moves count, accuracy percentage, bonus points for remaining time, and streak records.
  - Defeat / Timeout: Friendly encouraging prompt to try again with revealed cards.
- Integrate "Play Memory Mode" button in Rehearsal Setup alongside existing drill modes, with automatic selection of due or active cards from the selected stack.

## Acceptance Criteria
- [x] Time-attack mode with countdown options (e.g. 60s, 90s, 120s) and timeout defeat condition
- [x] Victory summary modal detailing elapsed time, move accuracy, speed score, and personal best records
- [ ] Memory Mode launch action integrated into the Rehearse setup screen and Stack detail view

## Design Notes
Store personal bests in application settings or stack rehearsal statistics. Connect navigation messages cleanly through `studyfrog-ui` without breaking existing flow.

## Dependencies
EPIC-059, STORY-0173, STORY-0174

## Implementation (2026-11-21)
The engine (`studyfrog-rehearsal::memory_summary`) provides the time-attack options (60, 90, 120 s), danger levels, the victory summary and the defeat line. The UI uses them:
- **Mode choice:** Stopwatch, 60 s, 90 s or 120 s, in the rehearse setup's What step. The choice is kept in the rehearse state and passed to the next game as the engine's time limit.
- **Danger colours:** the HUD time turns the warning colour under 30 s and the danger colour under 10 s of a countdown.
- **Summary:** a completed game opens a victory dialog (time, moves, accuracy, speed, best streak, the time bonus and the personal-best line) once, when the game is completed. A timeout shows the encouraging line under the HUD. Both have "Play again".
- **Records:** the best time of each stack in each vault is saved in `app.toml` as `[[memory_best]]` tables (`studyfrog-config::app`), so it survives restarts. The records live with the user's app settings, not in the vault, so a vault copied to another machine starts with no records.
- **Entry points:** "Memory game" in the rehearse setup, and beside "Start rehearsal" on the stack page.

Tests: `studyfrog-ui/tests/memory_mode.rs` (timeout, saved best, timing choice, stack start), `tests/memory_summary_ui.rs` (dialog and timeout text on screen) and `studyfrog-config/tests/memory_best_app.rs` (records round-trip through `app.toml`). Screenshots checked the timing row, the board, matched and mismatch cards, and the victory dialog.

## Open Questions
None.
