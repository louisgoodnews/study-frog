---
id: STORY-0023
type: story
title: "Runtime trait and implementation"
status: done
epic: EPIC-012
parents: [EPIC-012]
estimate: 8
labels: [m6, slm]
acceptance_criteria:
  - "`SlmRuntime` with a lexical implementation and a second one proving swappability"
  - "A semantic (candle) embedder is measured against the latency budget"
  - "Models are found, checked and reported when missing"
---

## Context
ADR 0004 asks for a measured runtime choice.

## Description
Runtime trait and implementation.

## Acceptance Criteria
- [x] `SlmRuntime` with a lexical implementation and a second one proving swappability
- [x] A semantic (candle) embedder is measured against the latency budget
- [x] Models are found, checked and reported when missing

## Design Notes
Children: TASK-0049 (done), TASK-0050 and TASK-0051 (blocked on the model download decision).

## Dependencies
None.

## Open Questions
None.
