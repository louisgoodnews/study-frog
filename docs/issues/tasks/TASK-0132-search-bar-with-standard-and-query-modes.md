---
id: TASK-0132
type: task
title: Search bar with Standard and Query modes
status: done
epic: EPIC-023
parents: [STORY-0057]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "The bar toggles between filter fields and a query box, each narrowing the list"
  - "Query mode shows errors under the box and completes field names; Standard converts to Query and back"
  - "The accessibility audit passes"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Build the bar on the parser and the list.

## Acceptance Criteria
- [x] The bar toggles between filter fields and a query box, each narrowing the list
- [x] Query mode shows errors under the box and completes field names; Standard converts to Query and back
- [x] The accessibility audit passes

## Design Notes
None.

## Dependencies
The parser and list tasks of this story.

## Open Questions
None.
