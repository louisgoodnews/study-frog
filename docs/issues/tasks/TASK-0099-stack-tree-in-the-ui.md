---
id: TASK-0099
type: task
title: "Stack tree in the library and in run setup"
status: done
epic: EPIC-020
parents: [STORY-0037]
estimate: 3
labels: [m8, ui]
acceptance_criteria:
  - "The stack list is indented by nesting and an open stack lists the stacks inside it"
  - "Ticking a stack in the run setup includes the stacks inside it"
---

## Context


## Description
Stack tree in the library and in run setup.

## Acceptance Criteria
- [x] The stack list is indented by nesting and an open stack lists the stacks inside it
- [x] Ticking a stack in the run setup includes the stacks inside it

## Design Notes
`SetupEdit::Stacks`. Items of a parent stack are only listed under the stack that owns them, not under its parent.

## Dependencies
TASK-0096.

## Open Questions
None.
