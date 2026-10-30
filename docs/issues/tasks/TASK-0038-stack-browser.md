---
id: TASK-0038
type: task
title: Stack browser
status: done
epic: EPIC-008
parents: [STORY-0017]
estimate: 3
labels: [m4, ui]
acceptance_criteria:
  - "List stacks and items with search by title"
  - "Create stack and card from the UI (writes Markdown via studyfrog-md)"
---

## Context
Reads from the index (`list_items`); writes go through new service use cases (`create_stack`, `create_card`) that render with `studyfrog-md`, never overwrite (numbered names), validate (single-line title, front and back required, a front containing a `???` line is refused), reindex immediately and re-prime the poll. New ids are ULIDs composed in-house from the clock and `/dev/urandom` (`compose_ulid`, `fresh_id_source`). Pure browser logic is in `ui/src/browse.rs`.

## Description
Stack browser.

## Acceptance Criteria
- [x] List stacks and items with search by title
- [x] Create stack and card from the UI (writes Markdown via studyfrog-md)

## Design Notes
The item list is a plain scroll area for now; virtualisation is deferred until a stack with thousands of items shows it is needed (the list is already filtered before drawing).

## Dependencies
TASK-0037.

## Open Questions
None.
