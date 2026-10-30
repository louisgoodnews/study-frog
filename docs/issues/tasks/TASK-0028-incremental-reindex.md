---
id: TASK-0028
type: task
title: Incremental reindex
status: backlog
epic: EPIC-006
parents: [STORY-0012]
estimate: 5
labels: [m2, store, perf]
acceptance_criteria:
  - "Content hash decides re-parse"
  - "Rename keeps history (id-based)"
  - "10k-note cold scan under the 3 s budget (measured)"
---

## Context
Reconcile files and index by id and content hash (ADR 0003).

## Description
Incremental reindex.

## Acceptance Criteria
- [ ] Content hash decides re-parse
- [ ] Rename keeps history (id-based)
- [ ] 10k-note cold scan under the 3 s budget (measured)

## Design Notes
Hash: choose by benchmark; do not add a crate if std hashing suffices for change detection.

## Dependencies
TASK-0027.

## Open Questions
None.
