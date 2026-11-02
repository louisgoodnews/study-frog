---
id: STORY-0064
type: story
title: Stack sub-views for list, backlog, roadmap, board, calendar and reports
status: done
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, ui, board, calendar, report]
acceptance_criteria:
  - "A stack has Sub-stacks, List, Backlog, Roadmap, Board, Calendar, Reports and Goals tabs, each scoped to the stack and its sub-stacks"
  - "Backlog separates what is up next from what is not yet planned, Roadmap shows items and stacks along a time axis"
  - "No tab loads its data until it is opened"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Fill the stack page's tabs with the Jira-style sub-views the owner asked for.

## Acceptance Criteria
- [x] A stack has Sub-stacks, List, Backlog, Roadmap, Board, Calendar, Reports and Goals tabs, each scoped to the stack and its sub-stacks
- [x] Backlog separates what is up next from what is not yet planned, Roadmap shows items and stacks along a time axis
- [x] No tab loads its data until it is opened

## Design Notes
List: the item list of STORY-0057 filtered to the stack, with the same filters and bulk actions. Backlog: two groups, 'Up next' (items scheduled for the coming days, with counts per status like Jira's coloured badges) and 'Not yet planned' (items with no schedule), with items moved between them by drag and drop. Roadmap: a time axis with zoom Today, Weeks, Months, Quarters; rows are sub-stacks and items with status and due date, bars run from the first to the next review or due date, and a line marks today. Board: the existing board scoped to the stack, grouped by status once statuses exist. Calendar: the stack's sessions and next reviews (STORY-0062). Reports: the stack's retention, answer time and familiarity reports. Sub-stacks: the whole tree of the stack with status, progress, due date and familiarity for each, folding open and closed. Goals: a bullet list the user writes and edits at any time (the owner, 2026-11-02), kept in the stack's own Markdown file under a Goals heading, so it is searchable and backed up; check boxes (`- [ ]`) are allowed. Forms (a Jira tab for collecting input from other people) is not built: StudyFrog has one user and the Create dialog and the importers already cover entering items.

Tasks: TASK-0152, TASK-0153, TASK-0154, TASK-0155, TASK-0156.

## Dependencies
STORY-0063 (page), STORY-0057 (list), STORY-0061 (statuses), STORY-0062 (calendar entries), STORY-0058 (familiarity).

## Open Questions
Is 'Backlog' the right name for items without a schedule, or would 'Not yet planned' do?
