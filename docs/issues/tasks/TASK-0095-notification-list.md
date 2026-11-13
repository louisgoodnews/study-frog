---
id: TASK-0095
type: task
title: "Notification list"
status: done
epic: EPIC-020
parents: [STORY-0036]
estimate: 2
labels: [m8, ui]
acceptance_criteria:
  - "Every new status notice is also kept in a list (last 30) shown by the bell, with a badge for the unseen ones"
---

## Context


## Description
Notification list.

## Acceptance Criteria
- [x] Every new status notice is also kept in a list (last 30) shown by the bell, with a badge for the unseen ones

## Design Notes
`chrome::track_notice` runs after every update; opening the bell marks them seen.

## Dependencies
TASK-0093.

## Open Questions
None.
