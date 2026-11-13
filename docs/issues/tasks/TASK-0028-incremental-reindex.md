---
id: TASK-0028
type: task
title: "Incremental reindex"
status: done
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
- [x] Content hash decides re-parse
- [x] Rename keeps history (id-based)
- [x] 10k-note cold scan under the 3 s budget (measured)

## Design Notes
In-house 64-bit hash (`hash.rs`), no crate: it only decides whether a file with a new mtime really changed. Unchanged files are stat-only. Measured (10k notes, in-memory DB, i9-11900K): scan+parse 282 ms, cold index 571 ms, warm reindex 43 ms vs the 3 s budget. Cached prepared statements cut the cold index from ~1.0 s to ~0.55 s.

## Dependencies
TASK-0027.

## Open Questions
None.
