---
id: TASK-0066
type: task
title: Profiling pass
status: done
epic: EPIC-017
parents: [STORY-0032]
estimate: 3
labels: [m8, bench]
acceptance_criteria:
  - "Every benchmark of the workspace run and compared with its budget"
  - "The one hot spot found is fixed and re-measured"
---

## Context


## Description
Profiling pass.

## Acceptance Criteria
- [x] Every benchmark of the workspace run and compared with its budget
- [x] The one hot spot found is fixed and re-measured

## Design Notes
Result in docs/contributing/BENCHMARKING.md. The lowest-recall queue computed a `powf` per comparison: computing it once per item took 10k items from 3.39 ms to 0.47 ms. `perf` is not installed on the reference machine, so the pass used the benchmarks and reasoning about the comparator rather than a sampling profile.

## Dependencies
None.

## Open Questions
None.
