---
id: TASK-0285
type: task
title: "dhat allocation counts for the rehearsal card, reports and console loops"
status: done
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
dhat is a workspace dependency, so no new crate. The three screens are drawn through the egui kittest harness
(`tests/common/views.rs`), headless, so no window is needed.

Each test is one whole app frame on its screen, repeated over 50 frames after 3 warm-up frames:
- `studyfrog-ui/tests/alloc_rehearsal_card.rs`: a two-card flip run on the Rehearse screen.
- `studyfrog-ui/tests/alloc_reports.rs`: the Reports screen on the Memory tab with the report bundle built.
- `studyfrog-ui/tests/alloc_console.rs`: the Console screen with five transcript lines.

Each file is its own test binary with one dhat profiler and a canary. `dhat` was added to studyfrog-ui's
dev-dependencies only; no unsafe code.

## Acceptance Criteria
- [x] A dhat test per render loop, or a recorded count
- [x] One test binary per profiler; no unsafe

## Measurements (before = after: no code changed)
Allocations per frame, measured twice each with identical results:

| screen | allocations in 50 frames | per frame | budget asserted |
|---|---|---|---|
| Console | 42,768 | 855.4 | 950 |
| Rehearsal card (flip run) | 46,360 | 927.2 | 1,030 |
| Reports (Memory tab) | 63,768 | 1,275.4 | 1,420 |

Zero is not reachable: egui allocates inside every frame (layout, shapes, text), so the budget is the
recorded count plus about 11%. A rise above a budget is a regression to review. The counts are for the
whole app frame on that screen (chrome included), not for the card alone.

## Design Notes
- All three loops allocate per frame. Reducing that is a separate task with a budget, if the owner wants it.
- The reports test uses a small fixed report bundle, so the count is for that data size.

## Dependencies
STORY-0205.

## Open Questions
- None for the measurement. Reducing the per-frame counts is not scheduled.
