---
id: TASK-0042
type: task
title: Force-directed layout (SoA)
status: done
epic: EPIC-009
parents: [STORY-0019]
estimate: 5
labels: [m4, ui, perf]
acceptance_criteria:
  - "Positions and velocities in struct-of-arrays form"
  - "Layout steps are deterministic given a seed"
---

## Context
Graph built from the `link` table.

## Description
Force-directed layout (SoA).

## Acceptance Criteria
- [x] Positions and velocities in struct-of-arrays form
- [x] Layout steps are deterministic given a seed

## Design Notes
New pure crate `studyfrog-graph` (deps: none; ADR 0001 amendment). Positions, velocities and force accumulators are separate `Vec<f32>` (SoA); repulsion uses a fixed uniform grid filled by a counting sort into preallocated arrays, so a step is roughly O(n) and allocates nothing. The repulsion window `(1 - d^2/cutoff^2)^2` takes the force smoothly to zero at the cutoff: with a hard cutoff the simulation never came to rest (measured 0.2-0.45 energy per node forever, even for unconnected nodes); with the window every tested shape reaches ~0 within 300-1000 steps. Seeded and deterministic (golden-angle spiral + SplitMix64 jitter, no rand crate). Tests: grid vs a brute-force oracle, determinism, springs, clustering, convergence, bounds (proptest).

## Dependencies
TASK-0037.

## Open Questions
None.
