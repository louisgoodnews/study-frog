---
id: TASK-0025
type: task
title: Initial schema
status: done
epic: EPIC-006
parents: [STORY-0011]
estimate: 3
labels: [m2, store]
acceptance_criteria:
  - "Tables for item index, links, reviews, mem state, runs, run events"
  - "WAL mode on; foreign keys on"
  - "Schema documented in docs/specs"
---

## Context
ADR 0003: rusqlite, bundled SQLite.

## Description
Initial schema.

## Acceptance Criteria
- [x] Tables for item index, links, reviews, mem state, runs, run events
- [x] WAL mode on; foreign keys on
- [x] Schema documented in docs/specs

## Design Notes
rusqlite 0.41 with bundled SQLite. Authoritative spec: docs/specs/storage-schema.md. Uids are stored as 26-char ULID text (debuggable, joins are cheap enough). History tables carry no foreign keys to `item`.

## Dependencies
None.

## Open Questions
None.
