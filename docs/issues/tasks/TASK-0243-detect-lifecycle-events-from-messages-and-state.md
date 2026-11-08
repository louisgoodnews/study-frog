---
id: TASK-0243
type: task
title: Detect lifecycle events from messages and state
status: done
epic: EPIC-028
parents: [STORY-0130]
estimate: 3
labels: [m12, ui]
acceptance_criteria:
  - "vault_opened, vault_closed, item_created, item_saved and rehearsal_finished are derived from the applied messages and a before/after probe of the state; unit tests"
---
## Context
Part of STORY-0130.

## Description
See the story and ADR 0013.

## Acceptance Criteria
- [x] vault_opened, vault_closed, item_created, item_saved and rehearsal_finished are derived from the applied messages and a before/after probe of the state; unit tests

## Design Notes
See the story.

## Dependencies
STORY-0130.

## Open Questions
None.
