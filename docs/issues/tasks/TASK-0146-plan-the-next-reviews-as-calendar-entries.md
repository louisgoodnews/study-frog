---
id: TASK-0146
type: task
title: Plan the next reviews as calendar entries
status: done
epic: EPIC-023
parents: [STORY-0062]
estimate: 3
labels: [m10, calendar, core]
acceptance_criteria:
  - "A pure function groups due dates into entries per stack and day with the item count and the stack's size"
  - "Complete and on-hold stacks and items produce no entries"
  - "Property tests cover grouping and skipping, and 10k items plan within a measured budget"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Implement the planner in studyfrog-calendar from the schedule of every item.

## Acceptance Criteria
- [x] A pure function groups due dates into entries per stack and day with the item count and the stack's size
- [x] Complete and on-hold stacks and items produce no entries
- [x] Property tests cover grouping and skipping, and 10k items plan within a measured budget

## Design Notes
None.

## Dependencies
The data model task of STORY-0061.

## Open Questions
None.
