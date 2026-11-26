---
id: TASK-0306
type: task
title: "SQLite migration and repository for exam plans, item links, and schedule blocks"
status: backlog
epic: EPIC-076
parents: ['STORY-0246']
estimate: 3
labels: [m14, store]
acceptance_criteria:
  - 'A new sequential SQLite migration in studyfrog-store creates tables for exam_plan, exam_target, and exam_schedule_block'
  - 'Transactional repository functions save, load, list, update, and delete exam plans and their linked targets/blocks'
  - 'Store integration tests verify full round-trip persistence across reopen and cascading deletes'
---

## Context
Part of `STORY-0246`. Persists exam schedules and item associations locally in the vault's SQLite index.

## Description
Add the next SQLite migration and `exam_plan` store module in `studyfrog-store` with atomic CRUD operations and round-trip tests.

## Acceptance Criteria
- [ ] A new sequential SQLite migration in `studyfrog-store` creates tables for `exam_plan`, `exam_target`, and `exam_schedule_block`
- [ ] Transactional repository functions save, load, list, update, and delete exam plans and their linked targets/blocks
- [ ] Store integration tests verify full round-trip persistence across reopen and cascading deletes

## Design Notes
- Keep migrations idempotent and follow existing `studyfrog-store` schema conventions.

## Dependencies
- `STORY-0246`, `TASK-0305`

## Open Questions
None.
