---
id: TASK-0022
type: task
title: Round-trip property test
status: backlog
epic: EPIC-005
parents: [STORY-0009]
estimate: 3
labels: [m2, md, testing]
acceptance_criteria:
  - "proptest generates items of every kind; parse(render(x)) == x"
  - "Regression seeds committed"
---

## Context
Guards the never-lose-user-data promise.

## Description
Round-trip property test.

## Acceptance Criteria
- [ ] proptest generates items of every kind; parse(render(x)) == x
- [ ] Regression seeds committed

## Design Notes
Generators live in md tests.

## Dependencies
TASK-0020, TASK-0021.

## Open Questions
None.
