---
id: TASK-0096
type: task
title: "Nested stacks and one creation use case"
status: done
epic: EPIC-020
parents: [STORY-0037]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "`service::create_more::create_item` creates a stack (top level or inside a stack), note, card or question, with tags; items without a stack are refused"
  - "The overview reports each stack parent and lists stacks in tree order"
  - "Counts and listings use the nearest stack, so inner items belong to the inner stack"
---

## Context


## Description
Nested stacks and one creation use case.

## Acceptance Criteria
- [x] `service::create_more::create_item` creates a stack (top level or inside a stack), note, card or question, with tags; items without a stack are refused
- [x] The overview reports each stack parent and lists stacks in tree order
- [x] Counts and listings use the nearest stack, so inner items belong to the inner stack

## Design Notes
A stack nests by living in a folder inside another stack folder (the existing ownership rule: nearest ancestor `_stack.md`); no format change. Existing orphan files still go to the Inbox.

## Dependencies
None.

## Open Questions
None.
