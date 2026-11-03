---
id: TASK-0164
type: task
title: Assistant proposals for user help pages
status: done
epic: EPIC-023
parents: [STORY-0066]
estimate: 3
labels: [m10, ui, agent]
acceptance_criteria:
  - "The assistant can read the user's help pages (shown first, sent only after Send, as for PDF pages) and propose a new page or an edit"
  - "A proposal changes nothing until the user confirms it, and supplied pages are never offered for change"
  - "The help area links to the assistant screen and back"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Extend the assistant's proposal changes to wiki pages and add the links.

## Acceptance Criteria
- [x] The assistant can read the user's help pages (shown first, sent only after Send, as for PDF pages) and propose a new page or an edit
- [x] A proposal changes nothing until the user confirms it, and supplied pages are never offered for change
- [x] The help area links to the assistant screen and back

## Design Notes
None.

## Dependencies
The user pages task of this story.

## Open Questions
None.
