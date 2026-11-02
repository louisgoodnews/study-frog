---
id: TASK-0155
type: task
title: Roadmap tab
status: backlog
epic: EPIC-023
parents: [STORY-0064]
estimate: 5
labels: [m10, ui, calendar]
acceptance_criteria:
  - "Rows show sub-stacks and items with status, and bars span their dates on a time axis with a today line"
  - "Zoom switches between today, weeks, months and quarters and the view scrolls horizontally"
  - "1000 rows draw inside the frame budget, measured"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Build the time-axis view as a pure layout function from rows and a zoom level plus a painter.

## Acceptance Criteria
- [ ] Rows show sub-stacks and items with status, and bars span their dates on a time axis with a today line
- [ ] Zoom switches between today, weeks, months and quarters and the view scrolls horizontally
- [ ] 1000 rows draw inside the frame budget, measured

## Design Notes
None.

## Dependencies
STORY-0063.

## Open Questions
None.
