---
id: EPIC-018
type: epic
title: "Service layer completion"
status: done
parents: []
labels: [m3, service, store]
acceptance_criteria:
  - "SM-2 is a working per-stack alternative to FSRS in the service"
  - "Daily new/review limits of each stack are honoured by spaced runs"
  - "A run's item filter is persisted with it"
  - "Stack settings are available from the index without parsing files"
---

## Context
Follow-up to M3: gaps left in studyfrog-service after the first end-to-end version (see STATUS.md).

## Description
Closes the three gaps (SM-2, budgets from stack limits, filter persistence) plus the stack settings they share.

## Acceptance Criteria
- [x] SM-2 is a working per-stack alternative to FSRS in the service
- [x] Daily new/review limits of each stack are honoured by spaced runs
- [x] A run's item filter is persisted with it
- [x] Stack settings are available from the index without parsing files

## Design Notes
Children: STORY-0034.

## Dependencies
M3 exit.

## Open Questions
None.
