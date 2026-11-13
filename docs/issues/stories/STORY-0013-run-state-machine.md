---
id: STORY-0013
type: story
title: "Run state machine"
status: done
epic: EPIC-007
parents: [EPIC-007]
estimate: 5
labels: [m3, rehearsal]
acceptance_criteria:
  - "Pure `step(RunState, Input) -> Result<(RunState, events)>`"
  - "Invalid inputs leave state unchanged"
---

## Context
Spec: docs/specs/rehearsal-run.md (state machine).

## Description
Run state machine.

## Acceptance Criteria
- [x] Pure `step(RunState, Input) -> Result<(RunState, events)>`
- [x] Invalid inputs leave state unchanged

## Design Notes
Children: TASK-0030, TASK-0031.

## Dependencies
Parent epic EPIC-007.

## Open Questions
None.
