---
id: EPIC-006
type: epic
title: Storage
status: done
parents: []
labels: [m2, store]
acceptance_criteria:
  - "SQLite schema with migrations; vault scanner keeps the index current"
  - "Renaming a file keeps its history (id-based)"
  - "Index can be dropped and rebuilt from files"
---

## Context
Storage split per ADR 0003: files authoritative for content, SQLite for history and derived index.

## Description
`studyfrog-store` is the only crate doing storage I/O.

## Acceptance Criteria
- [x] SQLite schema with migrations; vault scanner keeps the index current
- [x] Renaming a file keeps its history (id-based)
- [x] Index can be dropped and rebuilt from files

## Design Notes
Children: STORY-0011, STORY-0012.

## Dependencies
EPIC-005 (parsing).

## Open Questions
None.
