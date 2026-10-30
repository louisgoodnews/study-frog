---
id: STORY-0012
type: story
title: File layer and index sync
status: backlog
epic: EPIC-006
parents: [EPIC-006]
estimate: 8
labels: [m2, store]
acceptance_criteria:
  - "Vault scan parses and indexes 10k notes under budget"
  - "Incremental reindex by content hash"
  - "File changes picked up by a watcher"
---

## Context
Reconcile files and index by id and hash.

## Description
File layer and index sync.

## Acceptance Criteria
- [ ] Vault scan parses and indexes 10k notes under budget
- [ ] Incremental reindex by content hash
- [ ] File changes picked up by a watcher

## Design Notes
Children: TASK-0027, TASK-0028, TASK-0029.

## Dependencies
Parent epic EPIC-006.

## Open Questions
None.
