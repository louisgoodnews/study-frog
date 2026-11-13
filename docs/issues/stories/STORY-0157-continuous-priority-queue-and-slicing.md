---
id: STORY-0157
type: story
title: "Continuous Priority Queue and Anti-Paralysis Slicing"
status: done
epic: EPIC-052
parents: ['EPIC-052']
estimate: 5
labels: [m13, srs]
acceptance_criteria:
  - 'Effective priority function blends item priority, memory retrievability, and familiarity'
  - 'Queue builder slices overdue items to user-configured daily session budgets'
  - 'High-priority cards are protected from being starved by low-priority backlog mountains'
---

## Context
Overcoming spaced repetition review backlog burnout.

## Description
Implement continuous priority queue slicing: calculate effective priority from urgency, retrievability, and priority, truncating queues to user-defined limits.

## Acceptance Criteria
- [x] Effective priority function blends item priority, memory retrievability, and familiarity
- [x] Queue builder slices overdue items to user-configured daily session budgets
- [x] High-priority cards are protected from being starved by low-priority backlog mountains

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-052

## Open Questions
None.
