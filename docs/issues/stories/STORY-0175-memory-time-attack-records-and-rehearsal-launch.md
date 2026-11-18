---
id: STORY-0175
type: story
title: "Time-Attack Countdown Mode, Session Records, and Rehearsal Launch Entry Points"
status: backlog
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
- [ ] Time-attack mode with countdown options (e.g. 60s, 90s, 120s) and timeout defeat condition
- [ ] Victory summary modal detailing elapsed time, move accuracy, speed score, and personal best records
- [ ] Memory Mode launch action integrated into the Rehearse setup screen and Stack detail view

## Design Notes
Store personal bests in application settings or stack rehearsal statistics. Connect navigation messages cleanly through `studyfrog-ui` without breaking existing flow.

## Dependencies
EPIC-059, STORY-0173, STORY-0174

## Open Questions
None.
