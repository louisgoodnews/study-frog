---
id: STORY-0019
type: story
title: Link graph view
status: backlog
epic: EPIC-009
parents: [EPIC-009]
estimate: 8
labels: [m4, ui, perf]
acceptance_criteria:
  - "Force-directed layout in SoA form"
  - "5k nodes at 60 fps, zero allocation per frame (measured)"
---

## Context
Graph rendering is a benchmarked hot path.

## Description
Link graph view.

## Acceptance Criteria
- [ ] Force-directed layout in SoA form
- [ ] 5k nodes at 60 fps, zero allocation per frame (measured)

## Design Notes
Children: TASK-0042, TASK-0043.

## Dependencies
Parent epic EPIC-009.

## Open Questions
None.
