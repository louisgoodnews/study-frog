---
id: TASK-0049
type: task
title: SlmRuntime trait
status: done
epic: EPIC-012
parents: [STORY-0023]
estimate: 2
labels: [m6, slm]
acceptance_criteria:
  - "`SlmRuntime::embed` returning one vector per text"
  - "`SlmError` with Display, distinguishing an unavailable model from a failure"
---

## Context
The runtime must be replaceable.

## Description
SlmRuntime trait.

## Acceptance Criteria
- [x] `SlmRuntime::embed` returning one vector per text
- [x] `SlmError` with Display, distinguishing an unavailable model from a failure

## Design Notes
`studyfrog-slm::runtime`. A hand-made table runtime in the scoring tests proves a second implementation drives the same scoring.

## Dependencies
None.

## Open Questions
None.
