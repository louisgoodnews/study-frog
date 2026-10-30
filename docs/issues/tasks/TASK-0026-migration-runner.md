---
id: TASK-0026
type: task
title: Migration runner
status: backlog
epic: EPIC-006
parents: [STORY-0011]
estimate: 2
labels: [m2, store]
acceptance_criteria:
  - "Ordered SQL migrations applied transactionally"
  - "Applies on an empty DB and on the previous version"
  - "Refuses a DB newer than the app"
---

## Context
PRAGMA user_version.

## Description
Migration runner.

## Acceptance Criteria
- [ ] Ordered SQL migrations applied transactionally
- [ ] Applies on an empty DB and on the previous version
- [ ] Refuses a DB newer than the app

## Design Notes
Migrations as data (id, sql) slice; runner is a free function.

## Dependencies
TASK-0025.

## Open Questions
None.
