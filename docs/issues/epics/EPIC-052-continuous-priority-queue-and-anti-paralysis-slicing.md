---
id: EPIC-052
type: epic
title: "Continuous Priority Queue and Anti-Paralysis Slicing"
status: done
parents: []
labels: [m13, srs, rehearsal]
acceptance_criteria:
  - 'Effective priority function blends user card priority, retrievability gap, and familiarity'
  - 'Queue builder slices daily overdue queues to user-configured session budgets'
  - 'High-priority urgent cards are guaranteed review even under heavy overdue backlogs'
  - 'Review session summary provides cognitive burnout prevention pacing feedback'
---

## Context
Accumulated backlogs are the primary cause of spaced repetition system abandonment. Students faced with 500+ overdue cards experience cognitive paralysis.

## Description
Implement continuous priority queue slicing: combine numerical priority, memory retrievability, and familiarity into an effective priority score that bounds daily sessions to user budgets.

## Acceptance Criteria
- [x] Effective priority function blends user card priority, retrievability gap, and familiarity
- [x] Queue builder slices daily overdue queues to user-configured session budgets
- [x] High-priority urgent cards are guaranteed review even under heavy overdue backlogs
- [x] Review session summary provides cognitive burnout prevention pacing feedback

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
None.

## Open Questions
None.
