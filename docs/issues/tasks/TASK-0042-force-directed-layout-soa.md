---
id: TASK-0042
type: task
title: Force-directed layout (SoA)
status: backlog
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
- [ ] Positions and velocities in struct-of-arrays form
- [ ] Layout steps are deterministic given a seed

## Design Notes
Spatial grid or Barnes-Hut only if the benchmark requires it.

## Dependencies
TASK-0037.

## Open Questions
None.
