---
id: EPIC-007
type: epic
title: Rehearsal engine
status: ready
parents: []
labels: [m3, rehearsal]
acceptance_criteria:
  - "A scripted run produces the expected event log"
  - "Latency excludes paused time"
  - "Run results update SRS state and persist"
  - "Crashed runs recover as aborted"
---

## Context
Rehearsal turns the queue of due items into timed, measured practice (docs/specs/rehearsal-run.md).

## Description
Headless engine in `studyfrog-rehearsal`: state machine, latency capture, queue building, persistence of events.

## Acceptance Criteria
- [ ] A scripted run produces the expected event log
- [ ] Latency excludes paused time
- [ ] Run results update SRS state and persist
- [ ] Crashed runs recover as aborted

## Design Notes
Children: STORY-0013, STORY-0014, STORY-0015.

## Dependencies
M2 exit (parsing, storage). core run/event types exist (TASK-0015).

## Open Questions
Does queue building live here or in the shell (needs due data from the store)? Plan: pure builder here, store query in the shell.
