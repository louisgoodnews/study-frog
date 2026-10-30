---
id: TASK-0055
type: task
title: SLM benchmark
status: done
epic: EPIC-013
parents: [STORY-0025]
estimate: 1
labels: [m6, bench]
acceptance_criteria:
  - "A benchmark of comparing a 30-word answer"
  - "The result is recorded"
---

## Context
ADR 0004 sets a budget of p95 under 150 ms.

## Description
SLM benchmark.

## Acceptance Criteria
- [x] A benchmark of comparing a 30-word answer
- [x] The result is recorded

## Design Notes
`crates/studyfrog-slm/benches/compare.rs`: about 21 microseconds per comparison (lexical runtime, this machine). The semantic runtime must be measured the same way.

## Dependencies
TASK-0052.

## Open Questions
None.
