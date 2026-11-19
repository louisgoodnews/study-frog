---
id: STORY-0167
type: story
title: "Interactive Homework Planner Screen and Creation Dialog"
status: in_progress
epic: EPIC-056
parents: ['EPIC-056']
estimate: 5
labels: [m14, ui]
acceptance_criteria:
  - 'Dedicated Homework navigation tab/view accessible from main navigation rail'
  - 'Interactive assignment list with status filtering (All, Active, Due Today, Overdue, Completed)'
  - 'Modal dialog to create, edit, estimate, and complete homework tasks with course association'
---

## Context
Students require a frictionless and focused interface to manage their assignments, view upcoming tasks, and tick them off as completed.

## Description
Build the Homework management user interface in `studyfrog-ui`:
- Add a Homework view with clear visual hierarchy, subject badges, and due date relative countdowns.
- Provide quick-filter tabs for Active, Due Today, Overdue, and Completed assignments.
- Implement an accessible modal creation/edit dialog with subject selection, date-time picker, estimated duration slider, and reminder lead times.

## Acceptance Criteria
- [x] Dedicated Homework navigation tab/view accessible from main navigation rail
- [ ] Interactive assignment list with status filtering (All, Active, Due Today, Overdue, Completed)
- [ ] Modal dialog to create, edit, estimate, and complete homework tasks with course association

## Design Notes
Adhere strictly to StudyFrog design tokens, accessible names, keyboard navigation, and `palette_of(ui)` theming (no raw RGB).

## Dependencies
EPIC-056

## Open Questions
None.
