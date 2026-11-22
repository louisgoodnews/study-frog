---
id: STORY-0205
type: story
title: "Allocations in the render loops, counted with dhat, and a profile of the slowest screen"
status: backlog
epic: EPIC-067
parents: ['EPIC-067']
estimate: 3
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'Each render loop of the interactive screens has a dhat test, or a recorded allocation count per frame'
  - 'The graph layout, projection and picking loop asserts zero allocations over 50 frames (crates/studyfrog-graph/tests/allocs.rs)'
  - 'The slowest screen measured in STORY-0204 has a profile recorded in its task, before and after any change'
---

## Context
P7: no allocation in render loops, and profile before optimising. BENCHMARKING.md says a counting allocator needs
`unsafe`, which is forbidden, so allocations are counted with `dhat`.

## Description
- `dhat` is already a workspace dependency (used by the graph test), so no new crate is needed.
- Each further render loop gets a test, one per test binary, because dhat allows one profiler per process.
- The slowest screen gets a profile (`perf` or `cargo flamegraph`), which is not installed in this container.

## Acceptance Criteria
- [ ] Each render loop of the interactive screens has a dhat test or a recorded allocation count per frame (TASK-0285)
- [x] The graph layout, projection and picking loop asserts zero allocations over 50 frames (existing: crates/studyfrog-graph/tests/allocs.rs)
- [ ] The slowest screen has a profile recorded in its task (TASK-0286)

## Design Notes
- The existing graph test is the pattern: a canary allocation proves the profiler counts, then the loop must add zero.
- A loop that allocates is not fixed in this story; it becomes a task with its own budget and before/after numbers.

## Dependencies
TASK-0285, TASK-0286. TASK-0286 needs the slowest screen from STORY-0204.

## Open Questions
- Which render loops allocate on each frame today. Not measured yet.
