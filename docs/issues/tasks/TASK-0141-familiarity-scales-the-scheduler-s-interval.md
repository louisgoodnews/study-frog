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
  - "familiarity_factor and interval_days_with in studyfrog-srs follow docs/research/familiarity-and-fsrs.md and leave interval_days and the FSRS golden tests unchanged"
  - "Properties hold: strength 0 gives factor 1, a higher familiarity never lowers the factor, the factor stays inside its clamp and is 1 when familiarity equals the expected value, the interval is at least 1 day and at most the maximum, and nothing changes before the third review"
  - "The calendar planner and the queue builders use the adjusted interval, never past an exam or due date, and SM-2 uses the same factor with its interval as the stability"
  - "Settings hold the strength (default 2, on from the start) and a per-stack switch, and show the replay verdict of TASK-0184"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the scaling to the srs crate behind the schedule update and measure it on the simulated history.

## Acceptance Criteria
- [ ] familiarity_factor and interval_days_with in studyfrog-srs follow docs/research/familiarity-and-fsrs.md and leave interval_days and the FSRS golden tests unchanged
- [ ] Properties hold: strength 0 gives factor 1, a higher familiarity never lowers the factor, the factor stays inside its clamp and is 1 when familiarity equals the expected value, the interval is at least 1 day and at most the maximum, and nothing changes before the third review
- [ ] The calendar planner and the queue builders use the adjusted interval, never past an exam or due date, and SM-2 uses the same factor with its interval as the stability
- [ ] Settings hold the strength (default 2, on from the start) and a per-stack switch, and show the replay verdict of TASK-0184

## Design Notes
None.

## Dependencies
STORY-0058 and the research TASK-0183; TASK-0184 for the verdict shown beside the switch.

## Open Questions
None.
