---
id: TASK-0143
type: task
title: "Specify the statuses, their names and their transitions"
status: done
epic: EPIC-023
parents: [STORY-0061]
estimate: 2
labels: [m10, docs, decision]
acceptance_criteria:
  - "docs/specs/status.md lists each status with its stored value, label, meaning and allowed transitions"
  - "The owner has accepted the names and whether the board columns are bound to statuses"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Write the spec and get the owner's decision before any code.

## Acceptance Criteria
- [x] docs/specs/status.md lists each status with its stored value, label, meaning and allowed transitions
- [x] The owner has accepted the names and whether the board columns are bound to statuses

## Design Notes
Written 2026-11-02 in `docs/specs/status.md`. The five names are accepted by the owner. Proposed and awaiting the owner: the global kanban board keeps free-form columns, and a stack's Board tab groups by status (moving a card changes the status); the suggestion thresholds (0.8, 0.5, 0.95 for 30 days).

## Dependencies
None.

## Open Questions
Accepted by the owner on 2026-11-02: only a stack's Board tab is bound to statuses; the global board stays free-form.
