---
id: TASK-0026
type: task
title: Migration runner
status: done
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
- [x] Ordered SQL migrations applied transactionally
- [x] Applies on an empty DB and on the previous version
- [x] Refuses a DB newer than the app

## Design Notes
`Migration { version, name, sql }` slice in schema.rs; `migrate` is a free function. Each step runs in its own transaction together with the `user_version` bump; a failed step rolls back; a newer database is refused without being touched.

## Dependencies
TASK-0025.

## Open Questions
None.
