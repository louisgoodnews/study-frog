---
id: TASK-0286
type: task
title: "Profile the slowest screen and record the profile"
status: backlog
epic: EPIC-067
parents: ['STORY-0205']
estimate: 1
labels: [m14, perf]
acceptance_criteria:
  - 'The slowest screen from STORY-0204 has a profile (perf or cargo flamegraph) recorded in this task, naming its hot functions'
  - 'Any change made for it shows before and after numbers in its commit message (P7)'
---

## Context
Part of STORY-0205. EPIC-067 asks for a profile of the slowest screen.

## Description
Profile on the owner's machine. `perf` is not installed in this container, so no profile is recorded here.

## Acceptance Criteria
- [ ] A profile is recorded here, naming the hot functions
- [ ] Changes show before and after numbers in their commit

## Design Notes
- Profile before optimising (BENCHMARKING.md).

## Dependencies
STORY-0204 (which screen is slowest); a profiler on the owner's machine.

## Open Questions
None.
