---
id: TASK-0053
type: task
title: Scoring property tests
status: done
epic: EPIC-013
parents: [STORY-0024]
estimate: 2
labels: [m6, slm, test]
acceptance_criteria:
  - "Range, determinism, identity, empty answer and whitespace independence as properties"
  - "Ordering right > close > wrong on the lexical runtime"
---

## Context
The spec lists the properties.

## Description
Scoring property tests.

## Acceptance Criteria
- [x] Range, determinism, identity, empty answer and whitespace independence as properties
- [x] Ordering right > close > wrong on the lexical runtime

## Design Notes
`crates/studyfrog-slm/tests/score.rs`. Thresholds are not calibrated: that needs a labelled set and a semantic model.

## Dependencies
TASK-0052.

## Open Questions
None.
