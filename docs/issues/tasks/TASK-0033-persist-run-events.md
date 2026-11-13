---
id: TASK-0033
type: task
title: "Persist run events"
status: done
epic: EPIC-007
parents: [STORY-0014]
estimate: 3
labels: [m3, store, rehearsal]
acceptance_criteria:
  - "Append-only run_event rows, one transaction per item"
  - "Run, queue and state persisted; a run left Running is recovered as Aborted on start"
  - "Reviews and mem_state updated from ratings"
---

## Context
Schema exists (storage-schema.md).

## Description
Persist run events.

## Acceptance Criteria
- [x] Append-only run_event rows, one transaction per item
- [x] Run, queue and state persisted; a run left Running is recovered as Aborted on start
- [x] Reviews and mem_state updated from ratings

## Design Notes
Store: `runs.rs` (run, queue, append-only events with payloads, crash recovery that closes the log with `run_aborted`) and `schedule.rs` (reviews, FSRS memory state). Orchestration lives in the new `studyfrog-service` crate (ADR 0006): `record_step` persists events and, for `UpdateSchedule` plans only, writes a review and new memory state per rating, all in one transaction. SM-2 is not wired into the service yet (FSRS only).

## Dependencies
TASK-0031, TASK-0016.

## Open Questions
Resolved: a new application-service crate (ADR 0006, Proposed, flagged for owner review).
