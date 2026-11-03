---
id: TASK-0195
type: task
title: Calm the remaining screens
status: backlog
epic: EPIC-024
parents: [STORY-0073]
estimate: 5
labels: [m11, ui]
acceptance_criteria:
  - "Calendar, Board, Reports, Achievements, Assistant, Graph, the Create dialog and the sidebar follow the rules of docs/specs/ui-calm.md, checked with screenshots of the running app"
  - "The owner has looked at them"
---

## Context
The owner accepted the first part of the calmer UI on 2026-11-03 (library, stack page, Home, rehearsal setup, settings, help) and said that basically the whole application is affected.

## Description
Apply the same rules to the screens not yet done: a reading column or grouped cards, captions instead of large headings, aligned rows, one primary button per area, rare actions in a menu.

## Acceptance Criteria
- [ ] Calendar, Board, Reports, Achievements, Assistant, Graph, the Create dialog and the sidebar follow the rules of docs/specs/ui-calm.md, checked with screenshots of the running app
- [ ] The owner has looked at them

## Design Notes
Use `studyfrog --screenshot FILE --screen NAME` to look at each screen before and after. Open proposals in the spec: sidebar grouping, "More fields" in the Create dialog, folding the stack list.

## Dependencies
None.

## Open Questions
Whether the sidebar grouping and the folding stack list are wanted.
