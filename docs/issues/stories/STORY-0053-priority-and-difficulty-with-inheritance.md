---
id: STORY-0053
type: story
title: Priority and difficulty with inheritance
status: backlog
epic: EPIC-021
parents: [EPIC-021]
estimate: 8
labels: [m8, ui]
acceptance_criteria:
  - "Stacks, notes, cards and questions have a priority from lowest to highest"
  - "Stacks have a difficulty; items inherit the priority and difficulty of the parent stack unless they set their own"
  - "Priority affects the order and the suggestion rate in rehearsal runs"
---

## Context
The owner asked for this after reviewing the app on 2026-10-30.

## Description
Priority and difficulty with inheritance.

## Acceptance Criteria
- [ ] Stacks, notes, cards and questions have a priority from lowest to highest
- [ ] Stacks have a difficulty; items inherit the priority and difficulty of the parent stack unless they set their own
- [ ] Priority affects the order and the suggestion rate in rehearsal runs

## Design Notes
Stored in the front matter and the stack settings; used by the queue builder.

## Dependencies
None.

## Open Questions
None.
