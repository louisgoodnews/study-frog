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
- [ ] Victory summary modal detailing elapsed time, move accuracy, speed score, and personal best records
- [ ] Memory Mode launch action integrated into the Rehearse setup screen and Stack detail view

## Design Notes
Store personal bests in application settings or stack rehearsal statistics. Connect navigation messages cleanly through `studyfrog-ui` without breaking existing flow.

## Dependencies
EPIC-059, STORY-0173, STORY-0174

## Implementation (2026-11-21)
The engine (`studyfrog-rehearsal::memory_summary`) provides the time-attack options (60, 90, 120 s), danger levels, the victory summary and the defeat line. The UI now uses them:
- **Mode choice:** Stopwatch, 60 s, 90 s or 120 s, in the rehearse setup's What step. The choice is kept in the rehearse state and passed to the next game as the engine's time limit.
- **Danger colours:** the HUD time turns the warning colour under 30 s and the danger colour under 10 s of a countdown.
- **Summary:** a completed game shows the time, moves, accuracy, speed, best streak, the time bonus and the personal-best line; a timeout shows the encouraging line. Both have "Play again". The summary is an inline line under the HUD, not a modal.
- **Entry points:** "Memory game" in the rehearse setup, and "Memory game" beside "Start rehearsal" on the stack page.

Open: the personal best is kept for the session only (`Rehearse.memory_best_ms`). Keeping it across restarts needs a store or settings field, so the criterion about personal-best records is not ticked yet.

Tests: `studyfrog-ui/tests/memory_mode.rs` (timeout, best time, timing choice, stack start) and `tests/memory_summary_ui.rs` (summary and timeout text on screen). Screenshots checked the timing row and the board; the summary was checked by the harness, not visually.

## Open Questions
None.
