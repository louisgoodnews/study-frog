---
id: TASK-0212
type: task
title: Services follow the membership
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 8
labels: [m11, store]
acceptance_criteria:
  - "Queues, the stack summary, the roadmap, reports, the calendar planner, bulk edits and import and export use the membership: a referenced item counts in every stack that references it, once in a run and once in the global counts"
  - "Rules of the ADR hold: the primary stack's scheduler and limits decide the item's schedule"
  - "The library's stack column shows the primary stack and an 'Also in' column lists the others"
  - "For vaults without references every result is the same as before, checked by a test on the same vault"
  - "JSON export and import carry the references and CSV has the optional `also_in` column"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0. On the same day the owner decided about stacks: an item can be referenced by several stacks, held in a `referenced_by` field, with the original parent as the primary reference; stacks are not folders per se, but a folder can be marked as a 'stack' folder and is then also treated as a stack, while a 'non-stack' folder is a normal folder.

## Description
Everything that asks which items are in a stack asks the folder today; it must ask the membership.

## Acceptance Criteria
- [ ] Queues, the stack summary, the roadmap, reports, the calendar planner, bulk edits and import and export use the membership: a referenced item counts in every stack that references it, once in a run and once in the global counts
- [ ] Rules of the ADR hold: the primary stack's scheduler and limits decide the item's schedule
- [ ] The library's stack column shows the primary stack and an 'Also in' column lists the others
- [ ] For vaults without references every result is the same as before, checked by a test on the same vault
- [ ] JSON export and import carry the references and CSV has the optional `also_in` column

## Design Notes
Replace the folder-prefix checks (`stack_of`, `inside`) in the queue builders, the summary, the roadmap, the reports and the planner by the membership query of TASK-0211; the folder stays only the item's place on disk.

## Dependencies
TASK-0211.

## Open Questions
None.
