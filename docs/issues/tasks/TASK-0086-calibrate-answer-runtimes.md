---
id: TASK-0086
type: task
title: Calibrate the answer runtimes
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, followup]
acceptance_criteria:
  - "Each runtime reports a pivot and scores are stretched around it"
  - "A labelled set of 44 answers measures both runtimes"
---

## Context
An open flag from STATUS.md.

## Description
Calibrate the answer runtimes.

## Acceptance Criteria
- [x] Each runtime reports a pivot and scores are stretched around it
- [x] A labelled set of 44 answers measures both runtimes

## Design Notes
The earlier worry that 0.7 is too strict for the semantic runtime came from 10 trivial pairs and was wrong; word matching was the one that needed a pivot (0.44). Wrong-fact answers cannot be told apart by similarity (documented).

## Dependencies
None.

## Open Questions
None.
