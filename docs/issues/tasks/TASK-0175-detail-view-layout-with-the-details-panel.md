---
id: TASK-0175
type: task
title: Detail view layout with the details panel
status: backlog
epic: EPIC-023
parents: [STORY-0069]
estimate: 5
labels: [m10, ui]
acceptance_criteria:
  - "The content sits on the left and the Details panel on the right, for a stack and for a card, note and question"
  - "Every field of the current item page and of the old stack page is still editable, in place and saved as before"
  - "A stack shows its child items, PDFs and links, and the page passes the accessibility audit"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Rework view/editor.rs, view/item_details.rs and the stack page to the new layout.

## Acceptance Criteria
- [ ] The content sits on the left and the Details panel on the right, for a stack and for a card, note and question
- [ ] Every field of the current item page and of the old stack page is still editable, in place and saved as before
- [ ] A stack shows its child items, PDFs and links, and the page passes the accessibility audit

## Design Notes
None.

## Dependencies
The specification task of this story.

## Open Questions
None.
