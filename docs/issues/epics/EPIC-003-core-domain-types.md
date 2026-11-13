---
id: EPIC-003
type: epic
title: "Core domain types"
status: done
parents: []
labels: [m1, core]
acceptance_criteria:
  - "`studyfrog-core` defines ids, arenas, Stack, Note, Card, Question, Review, Run and event types as data only"
  - "No inherent impl blocks (repo-lint passes)"
---

## Context
Domain data model is the root of the dependency graph and must be stable before storage and UI.

## Description
Data-only domain types for the whole product.

## Acceptance Criteria
- [x] `studyfrog-core` defines ids, arenas, Stack, Note, Card, Question, Review, Run and event types as data only
- [x] No inherent impl blocks (repo-lint passes)

## Design Notes
Children: STORY-0005, STORY-0006. SoA/arena layouts per P3.

## Dependencies
M0 exit.

## Open Questions
Which fields of Run/Event are hot enough for SoA (decide with the M3 benchmark)?
