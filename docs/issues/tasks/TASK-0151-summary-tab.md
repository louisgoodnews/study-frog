---
id: TASK-0151
type: task
title: "Summary tab"
status: done
epic: EPIC-023
parents: [STORY-0063]
estimate: 5
labels: [m10, ui, report]
acceptance_criteria:
  - "Four cards show completed, updated, created and due soon for 7 days"
  - "A ring chart shows the status overview, a bar chart the priorities and a table the types, each with values in text"
  - "An activity feed lists recent changes, with an empty state"
  - "A Sub-stacks section lists the sub-stacks with status, progress and familiarity"
  - "Nothing loads until the tab opens"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the data functions and the Summary view.

## Acceptance Criteria
- [x] Four cards show completed, updated, created and due soon for 7 days
- [x] A ring chart shows the status overview, a bar chart the priorities and a table the types, each with values in text
- [x] An activity feed lists recent changes, with an empty state
- [x] A Sub-stacks section lists the sub-stacks with status, progress and familiarity
- [x] Nothing loads until the tab opens

## Design Notes
None.

## Dependencies
The shell task and STORY-0061.

## Open Questions
None.
