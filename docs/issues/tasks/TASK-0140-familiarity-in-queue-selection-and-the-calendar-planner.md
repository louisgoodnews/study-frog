---
id: TASK-0140
type: task
title: Familiarity in queue selection and the calendar planner
status: backlog
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
- [ ] Under a daily budget, due items are chosen by lower familiarity first, and items at or above the threshold are left out
- [ ] The calendar planner plans the next review of high-familiarity items later, never past an exam or due date
- [ ] Settings hold the threshold

## Design Notes
None.

## Dependencies
STORY-0058 and the planner task of STORY-0062.

## Open Questions
None.
