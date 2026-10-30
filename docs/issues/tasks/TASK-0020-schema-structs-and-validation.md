---
id: TASK-0020
type: task
title: Schema structs and validation
status: ready
epic: EPIC-005
parents: [STORY-0009]
estimate: 3
labels: [m2, md]
acceptance_criteria:
  - "Typed front-matter structs per kind (stack, note, card, question) with defaults"
  - "Validation errors carry file and key; duplicate ids detected"
  - "Unknown keys preserved in a remainder map"
---

## Context
Spec: docs/specs/markdown-schema.md.

## Description
Schema structs and validation.

## Acceptance Criteria
- [ ] Typed front-matter structs per kind (stack, note, card, question) with defaults
- [ ] Validation errors carry file and key; duplicate ids detected
- [ ] Unknown keys preserved in a remainder map

## Design Notes
Data structs in md, validation as free functions; conversion into core types.

## Dependencies
None.

## Open Questions
Where do remainder keys live in core types (side table keyed by Uid vs field)?
