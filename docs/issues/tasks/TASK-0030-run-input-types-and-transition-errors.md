---
id: TASK-0030
type: task
title: "Run input types and transition errors"
status: done
epic: EPIC-007
parents: [STORY-0013]
estimate: 2
labels: [m3, rehearsal, core]
acceptance_criteria:
  - "Input enum with monotonic timestamp (Start, Flip, Skip, SubmitAnswer, Rate, Pause, Resume, Abort); showing the next item and finishing are automatic"
  - "Typed transition error for inputs invalid in the current state"
---

## Context
Event and run types already landed in core (TASK-0015); this task adds the engine's input and error types (re-scoped from 'Run event types').

## Description
Run input types and transition errors.

## Acceptance Criteria
- [x] Input enum with monotonic timestamp (Start, Flip, Skip, SubmitAnswer, Rate, Pause, Resume, Abort); showing the next item and finishing are automatic
- [x] Typed transition error for inputs invalid in the current state

## Design Notes
`Action`/`Input` (timestamped), `StepOutput`, `StepError` (9 variants, hand-written Display delegating to a free function). `Session` holds the run, working queue, cursor, item phase and timers.

## Dependencies
TASK-0015.

## Open Questions
None.
