---
id: TASK-0141
type: task
title: "Familiarity scales the scheduler's interval"
status: done
epic: EPIC-023
parents: [STORY-0059]
estimate: 5
labels: [m10, srs]
acceptance_criteria:
  - "familiarity_factor and interval_days_with in studyfrog-srs follow docs/research/familiarity-and-fsrs.md and leave interval_days and the FSRS golden tests unchanged"
  - "Properties hold: strength 0 gives factor 1, a higher familiarity never lowers the factor, the factor stays inside its clamp and is 1 when familiarity equals the expected value, the interval is at least 1 day and at most the maximum, and nothing changes before the third review"
  - "The due date set after each rating uses the adjusted interval, never stretched past an exam or deadline, and SM-2 uses the same factor with its interval as the stability"
  - "Settings show the strength (default 2, on from the start) and the vault file and each stack can set it, with a per-stack switch; the replay verdict of TASK-0184 is shown beside it when that task lands"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the scaling to the srs crate behind the schedule update and measure it on the simulated history.

## Acceptance Criteria
- [x] familiarity_factor and interval_days_with in studyfrog-srs follow docs/research/familiarity-and-fsrs.md and leave interval_days and the FSRS golden tests unchanged
- [x] Properties hold: strength 0 gives factor 1, a higher familiarity never lowers the factor, the factor stays inside its clamp and is 1 when familiarity equals the expected value, the interval is at least 1 day and at most the maximum, and nothing changes before the third review
- [x] The due date set after each rating uses the adjusted interval, never stretched past an exam or deadline, and SM-2 uses the same factor with its interval as the stability
- [x] Settings show the strength (default 2, on from the start) and the vault file and each stack can set it, with a per-stack switch; the replay verdict of TASK-0184 is shown beside it when that task lands

## Design Notes
Done: `studyfrog-srs::familiarity` and `studyfrog-service::familiarity`, called from `rate_item` after each rating; see `docs/specs/srs-algorithm.md`. The calendar planner (TASK-0146) and the queue builders read the stored due dates, which already include the factor, so they need no change of their own; the deadline cut is done when the due date is set. The replay verdict beside the switch comes with TASK-0184. Rating speed and the exam date for the fall are not used yet (see the specification).

Updated 2026-11-02: the answer time and expected reading time of each rating, the reading time of each view and the exam or due date now reach the factor (migration 17, `StepContext.timing`, `RecordView.expected_ms`).

## Dependencies
STORY-0058 and the research TASK-0183; TASK-0184 for the verdict shown beside the switch.

## Open Questions
None.
