---
id: TASK-0150
type: task
title: Stack page shell with header and tabs
status: done
epic: EPIC-023
parents: [STORY-0063]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "A stack opens with a header, a tab bar with an overflow menu and the last tab remembered per stack"
  - "Existing content (description, PDFs, items, due date) is reachable from a tab or from the stack detail view"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Build the shell on the shared tab widget and move the existing content into tabs.

## Acceptance Criteria
- [x] A stack opens with a header, a tab bar with an overflow menu and the last tab remembered per stack
- [x] Existing content (description, PDFs, items, due date) is reachable from a tab or from the stack detail view

## Design Notes
None.

## Dependencies
The specification task of this story and STORY-0060.

## Open Questions
None.
