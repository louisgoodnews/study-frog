---
id: TASK-0130
type: task
title: Search query parser and evaluator
status: backlog
epic: EPIC-023
parents: [STORY-0057]
estimate: 5
labels: [m10, core, store]
acceptance_criteria:
  - "A pure parser turns a query into a syntax tree and reports errors with a position and the expected token"
  - "The evaluator runs the tree on the index with bound parameters; a property test shows no input reaches anything but item fields"
  - "Fuzzing finds no panic, and 10k items filter within the budget in the spec, measured"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Implement the grammar from the spec as a hand-written parser and an evaluator over the index.

## Acceptance Criteria
- [ ] A pure parser turns a query into a syntax tree and reports errors with a position and the expected token
- [ ] The evaluator runs the tree on the index with bound parameters; a property test shows no input reaches anything but item fields
- [ ] Fuzzing finds no panic, and 10k items filter within the budget in the spec, measured

## Design Notes
None.

## Dependencies
The specification task of this story.

## Open Questions
None.
