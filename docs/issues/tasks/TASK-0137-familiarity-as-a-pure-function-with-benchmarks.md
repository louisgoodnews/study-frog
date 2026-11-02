---
id: TASK-0137
type: task
title: Familiarity as a pure function with benchmarks
status: backlog
epic: EPIC-023
parents: [STORY-0058]
estimate: 5
labels: [m10, srs, report]
acceptance_criteria:
  - "A pure function returns the value in 0.0 to 1.0 and its confidence from the history, and a stack's value from its items"
  - "Properties hold: a view never lowers it, an easier rating raises it more than a harder one, a harder rating lowers it, a shorter view never gives a lower value than a longer one, and without contact the value never rises and falls only while the item is not Complete and its exam date has not passed"
  - "10k items are valued within the budget in the spec, measured with a benchmark"
  - "Error on the simulated history is recorded in docs/specs/reports.md"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Implement the formula from the spec in the report crate with property tests, a benchmark and the validation numbers.

## Acceptance Criteria
- [ ] A pure function returns the value in 0.0 to 1.0 and its confidence from the history, and a stack's value from its items
- [ ] Properties hold: a view never lowers it, an easier rating raises it more than a harder one, a harder rating lowers it, a shorter view never gives a lower value than a longer one, and without contact the value never rises and falls only while the item is not Complete and its exam date has not passed
- [ ] 10k items are valued within the budget in the spec, measured with a benchmark
- [ ] Error on the simulated history is recorded in docs/specs/reports.md

## Design Notes
None.

## Dependencies
The specification task of this story.

## Open Questions
None.
