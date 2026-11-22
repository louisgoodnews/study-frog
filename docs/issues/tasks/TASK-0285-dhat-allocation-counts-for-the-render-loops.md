---
id: TASK-0285
type: task
title: "dhat allocation counts for the rehearsal card, reports and console loops"
status: backlog
epic: EPIC-067
parents: ['STORY-0205']
estimate: 2
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'Each render loop of the rehearsal card, reports and console has a dhat test that asserts zero allocations over a fixed number of frames, or records the count'
  - 'Each test is its own test binary, and no unsafe code is added'
---

## Context
Part of STORY-0205. The graph loop already has the test (crates/studyfrog-graph/tests/allocs.rs).

## Description
Follow the graph test's pattern: a canary allocation proves the profiler counts, then the loop must add zero.
dhat is a workspace dependency, so no new crate.

## Acceptance Criteria
- [ ] A dhat test per render loop, or a recorded count
- [ ] One test binary per profiler; no unsafe

## Design Notes
- Which loops allocate is not known yet; a loop that does is a new task with a budget.

## Dependencies
STORY-0205.

## Open Questions
- How to call the render loops without a window (they may need the egui context).
