---
id: TASK-0212
type: task
title: Services follow the new membership
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 8
labels: [m11, store]
acceptance_criteria:
  - "Queues, the stack summary, the roadmap, reports, the calendar planner, bulk moves, the PDF notes and import and export use the membership and give the same results as before for migrated vaults"
  - "A test compares the results of old and new layout on the same vault"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Everything that asks which items are in a stack today asks the folder; it must ask the membership.

## Acceptance Criteria
- [ ] Queues, the stack summary, the roadmap, reports, the calendar planner, bulk moves, the PDF notes and import and export use the membership and give the same results as before for migrated vaults
- [ ] A test compares the results of old and new layout on the same vault

## Design Notes
Find every use of the stack folder as membership (`stack_of`, `inside`, folder prefix checks) and replace it with the membership query; keep the folder as the file's place only.

## Dependencies
TASK-0211.

## Open Questions
None.
