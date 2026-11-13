---
id: TASK-0079
type: task
title: "Persist sessions"
status: done
epic: EPIC-010
parents: [STORY-0020]
estimate: 3
labels: [m5, store, service]
acceptance_criteria:
  - "Schema v4 stores sessions and cancelled dates"
  - "Create, edit, delete and list sessions through the service"
  - "Sessions survive a restart; deleting a stack does not delete its sessions (the link just becomes empty)"
---

## Context
ADR 0003: calendar data is authoritative in SQLite.

## Description
Persist sessions.

## Acceptance Criteria
- [x] Schema v4 stores sessions and cancelled dates
- [x] Create, edit, delete and list sessions through the service
- [x] Sessions survive a restart; deleting a stack does not delete its sessions (the link just becomes empty)

## Design Notes
Schema v4 `session` + `session_cancelled` (CHECK constraints on times, durations, repeat and end kinds); `studyfrog-store::sessions`; service `planning` (validated save, delete, cancel one occurrence). Sessions reference stacks by uid without a foreign key, so removing a stack never deletes sessions. Survives a restart (tested).

## Dependencies
TASK-0044.

## Open Questions
None.
