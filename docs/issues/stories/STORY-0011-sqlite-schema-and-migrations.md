---
id: STORY-0011
type: story
title: SQLite schema and migrations
status: done
epic: EPIC-006
parents: [EPIC-006]
estimate: 5
labels: [m2, store]
acceptance_criteria:
  - "Initial schema for reviews, runs, events, index"
  - "Ordered migrations tracked with PRAGMA user_version"
---

## Context
ADR 0003.

## Description
SQLite schema and migrations.

## Acceptance Criteria
- [x] Initial schema for reviews, runs, events, index
- [x] Ordered migrations tracked with PRAGMA user_version

## Design Notes
Children: TASK-0025, TASK-0026.

## Dependencies
Parent epic EPIC-006.

## Open Questions
None.
