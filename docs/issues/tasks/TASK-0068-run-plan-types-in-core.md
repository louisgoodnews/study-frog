---
id: TASK-0068
type: task
title: Run plan types in core
status: done
epic: EPIC-007
parents: [STORY-0033]
estimate: 3
labels: [m3, core]
acceptance_criteria:
  - "`RunPlan { order, filter, effect, seed, limit, requeue_again }` and its parts are plain data in core"
  - "Constructors for the standard plans: spaced, sequential, shuffled, hardest, stalest"
  - "Persisted names round-trip; `Run` carries its plan"
---

## Context
Plans are persisted with runs, so they belong in core.

## Description
Run plan types in core.

## Acceptance Criteria
- [x] `RunPlan { order, filter, effect, seed, limit, requeue_again }` and its parts are plain data in core
- [x] Constructors for the standard plans: spaced, sequential, shuffled, hardest, stalest
- [x] Persisted names round-trip; `Run` carries its plan

## Design Notes
Data plus free functions only. `ItemKind` lives in core (md has its own `Kind` that includes stacks). Defaults: spaced = Due/UpdateSchedule/requeue; sequential, shuffled, stalest = PracticeOnly/no requeue; hardest = PracticeOnly/requeue. `MAX_REQUEUES` = 3. `Run` now carries its `plan`.

## Dependencies
TASK-0015.

## Open Questions
None.
