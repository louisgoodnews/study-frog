---
id: TASK-0063
type: task
title: Define extension points
status: done
epic: EPIC-016
parents: [STORY-0030]
estimate: 3
labels: [m8, ext]
acceptance_criteria:
  - "`Importer` (text of a file to cards) and `CardType` (source text to cards) records in the dependency-free crate `studyfrog-ext`"
  - "Contracts documented: pure functions, errors as messages, the application validates and writes the cards"
---

## Context


## Description
Define extension points.

## Acceptance Criteria
- [x] `Importer` (text of a file to cards) and `CardType` (source text to cards) records in the dependency-free crate `studyfrog-ext`
- [x] Contracts documented: pure functions, errors as messages, the application validates and writes the cards

## Design Notes
`studyfrog-ext::types`; docs/EXTENDING.md; ADR 0008 (Proposed).

## Dependencies
None.

## Open Questions
None.
