---
id: TASK-0022
type: task
title: "Round-trip property test"
status: done
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
- [x] proptest generates items of every kind; parse(render(x)) == x
- [x] Regression seeds committed

## Design Notes
`tests/roundtrip.rs`: 1000 generated documents of all four kinds; `parse(render(x)) == x`, rendering is stable, parser never panics on arbitrary input. Documents whose front text would swallow the `???` separator are rejected at render time (`AmbiguousSeparator`) instead of corrupting the file.

## Dependencies
TASK-0020, TASK-0021.

## Open Questions
None.
