---
id: STORY-0024
type: story
title: Scoring contract
status: done
epic: EPIC-013
parents: [EPIC-013]
estimate: 5
labels: [m6, slm]
acceptance_criteria:
  - "Pure score functions over embeddings per docs/specs/scoring.md"
  - "Property tests for range, identity, emptiness, whitespace and determinism"
---

## Context
Scoring must not depend on the runtime.

## Description
Scoring contract.

## Acceptance Criteria
- [x] Pure score functions over embeddings per docs/specs/scoring.md
- [x] Property tests for range, identity, emptiness, whitespace and determinism

## Design Notes
Children: TASK-0052, TASK-0053.

## Dependencies
TASK-0049.

## Open Questions
None.
