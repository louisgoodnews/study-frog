---
id: TASK-0025
type: task
title: Initial schema
status: backlog
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
- [ ] Tables for item index, links, reviews, mem state, runs, run events
- [ ] WAL mode on; foreign keys on
- [ ] Schema documented in docs/specs

## Design Notes
Add `rusqlite` at first use.

## Dependencies
None.

## Open Questions
None.
