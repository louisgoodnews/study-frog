---
id: TASK-0013
type: task
title: Stack, Note, Card, Question structs
status: ready
epic: EPIC-003
parents: [STORY-0005]
estimate: 3
labels: [m1, core]
acceptance_criteria:
  - "Plain structs matching docs/specs/markdown-schema.md fields"
  - "Each has a `uid: Uid`; cross references use `Uid` (persisted) or `Idx<T>` (in memory)"
  - "No behaviour blocks"
---

## Context
Item kinds from the glossary.

## Description
Stack, Note, Card, Question structs.

## Acceptance Criteria
- [ ] Plain structs matching docs/specs/markdown-schema.md fields
- [ ] Each has a `uid: Uid`; cross references use `Uid` (persisted) or `Idx<T>` (in memory)
- [ ] No behaviour blocks

## Design Notes
Data only; no parsing here.

## Dependencies
TASK-0012.

## Open Questions
Is Item a sum type (enum) or three separate arenas? Prefer separate arenas (SoA); confirm during implementation.
