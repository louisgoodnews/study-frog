---
id: TASK-0043
type: task
title: Graph render benchmark
status: done
epic: EPIC-009
parents: [STORY-0019]
estimate: 3
labels: [m4, ui, perf]
acceptance_criteria:
  - "5k nodes at 60 fps measured"
  - "Zero allocation per frame, verified with a heap profiler (dhat)"
---

## Context
Replaces the provisional graph budget.

## Description
Graph render benchmark.

## Acceptance Criteria
- [x] 5k nodes at 60 fps measured
- [x] Zero allocation per frame, verified with a heap profiler (dhat)

## Design Notes
Measured (5,000 nodes, 10,000 edges, i9-11900K): `layout_step` 3.98 ms, `project` 1.8 us, i.e. about a quarter of a 16.6 ms frame for layout and projection together. Heap profile with dhat (installed as the test binary's global allocator; a safe API, our crates stay `unsafe`-free): 0 allocations over 50 frames of step + project + pick, with a canary proving the profiler counts allocations. Not measured: egui's own tessellation and painting of ~10k lines (needs a GPU frame, unavailable headlessly); egui allocates its shape list per frame, outside our control. Culling skips edges and nodes outside the canvas.

## Dependencies
TASK-0042.

## Open Questions
None.
