---
id: TASK-0140
type: task
title: "Familiarity in queue selection and the calendar planner"
status: done
epic: EPIC-023
parents: [STORY-0059]
estimate: 3
labels: [m10, srs, calendar, rehearsal]
acceptance_criteria:
  - "Under a daily budget, due items are chosen by lower familiarity first, and items at or above the threshold are left out"
  - "The calendar planner plans the next review of high-familiarity items later, never past an exam or due date"
  - "Settings hold the threshold"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add familiarity to the queue builders and the calendar planner.

## Acceptance Criteria
- [x] Under a daily budget, due items are chosen by lower familiarity first, and items at or above the threshold are left out
- [x] The calendar planner plans the next review of high-familiarity items later, never past an exam or due date
- [x] Settings hold the threshold

## Design Notes
None.

## Dependencies
STORY-0058 and the planner task of STORY-0062.

## Open Questions
None.

## Outcome
The planner needs no extra rule: the due date of an item already carries the familiarity factor of TASK-0141, which cuts a stretched interval at the stack's exam or due date, and the calendar planner (TASK-0146) plans from those due dates. The threshold is `familiarity_threshold` in `[srs]` of `studyfrog.toml` (default 1, which leaves nothing out; the owner may want a lower default). Run order `least_familiar` and a "less familiar than N %" filter are stored with the run (migration 19).
