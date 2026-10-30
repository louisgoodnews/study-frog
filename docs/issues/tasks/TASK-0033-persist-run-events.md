---
id: TASK-0033
type: task
title: Persist run events
status: backlog
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
- [ ] Append-only run_event rows, one transaction per item
- [ ] Run, queue and state persisted; a run left Running is recovered as Aborted on start
- [ ] Reviews and mem_state updated from ratings

## Design Notes
Store functions take plain data from core; no dependency from rehearsal on store.

## Dependencies
TASK-0031, TASK-0016.

## Open Questions
Where to compose rehearsal + srs + store: the shell (ui/app). Confirm in implementation.
