---
id: TASK-0015
type: task
title: "Run and event types"
status: done
epic: EPIC-003
parents: [STORY-0006]
estimate: 3
labels: [m1, core, rehearsal]
acceptance_criteria:
  - "Run, RunEvent variants per docs/specs/rehearsal-run.md"
  - "Latency in integer ms"
---

## Context
Rehearsal engine input and output.

## Description
Run and event types.

## Acceptance Criteria
- [x] Run, RunEvent variants per docs/specs/rehearsal-run.md
- [x] Latency in integer ms

## Design Notes
Data only. Events are AoS (`RunEvent` with `EventKind` enum); SoA deferred to the M3 benchmark. `t_ms`/latency are `u32` ms. Persisted names via free functions (`event_name`, `run_state_name`, ...).

## Dependencies
TASK-0012.

## Open Questions
SoA vs AoS for the event log: decide with the TASK-0035 benchmark.
