---
id: TASK-0030
type: task
title: Run input types and transition errors
status: ready
epic: EPIC-007
parents: [STORY-0013]
estimate: 2
labels: [m3, rehearsal, core]
acceptance_criteria:
  - "Input enum (Start, Show, Flip, Skip, SubmitAnswer, Rate, Pause, Resume, Finish, Abort)"
  - "Typed transition error for inputs invalid in the current state"
---

## Context
Event and run types already landed in core (TASK-0015); this task adds the engine's input and error types (re-scoped from 'Run event types').

## Description
Run input types and transition errors.

## Acceptance Criteria
- [ ] Input enum (Start, Show, Flip, Skip, SubmitAnswer, Rate, Pause, Resume, Finish, Abort)
- [ ] Typed transition error for inputs invalid in the current state

## Design Notes
Data only; lives in studyfrog-rehearsal.

## Dependencies
TASK-0015.

## Open Questions
None.
