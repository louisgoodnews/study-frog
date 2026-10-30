---
id: TASK-0029
type: task
title: File watcher
status: backlog
epic: EPIC-006
parents: [STORY-0012]
estimate: 3
labels: [m2, store]
acceptance_criteria:
  - "Create/modify/delete/rename events update the index"
  - "Burst of events coalesced"
---

## Context
Keeps the index fresh while the app runs.

## Description
File watcher.

## Acceptance Criteria
- [ ] Create/modify/delete/rename events update the index
- [ ] Burst of events coalesced

## Design Notes
Evaluate `notify` vs inotify directly (P10); decide in the task.

## Dependencies
TASK-0028.

## Open Questions
None.
