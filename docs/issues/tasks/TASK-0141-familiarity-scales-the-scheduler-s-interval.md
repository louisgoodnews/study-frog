---
id: TASK-0141
type: task
title: Familiarity scales the scheduler's interval
status: backlog
epic: EPIC-023
parents: [STORY-0059]
estimate: 5
labels: [m10, srs]
acceptance_criteria:
  - "A pure function scales the interval that FSRS or SM-2 computed by a multiplier that rises with familiarity, within the lower and upper bound of the spec and never past an exam or due date"
  - "Properties hold: higher familiarity never gives a shorter interval, strength 0 returns the algorithm's interval, and bounds hold for every input"
  - "A simulated history shows how retention and workload change with the strength, the numbers are recorded in docs/specs/reports.md and the default strength follows from them"
  - "Settings hold the strength, and the golden tests of FSRS and SM-2 still pass with strength 0"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the scaling to the srs crate behind the schedule update and measure it on the simulated history.

## Acceptance Criteria
- [ ] A pure function scales the interval that FSRS or SM-2 computed by a multiplier that rises with familiarity, within the lower and upper bound of the spec and never past an exam or due date
- [ ] Properties hold: higher familiarity never gives a shorter interval, strength 0 returns the algorithm's interval, and bounds hold for every input
- [ ] A simulated history shows how retention and workload change with the strength, the numbers are recorded in docs/specs/reports.md and the default strength follows from them
- [ ] Settings hold the strength, and the golden tests of FSRS and SM-2 still pass with strength 0

## Design Notes
None.

## Dependencies
STORY-0058.

## Open Questions
None.
