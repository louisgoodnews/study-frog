---
id: TASK-0186
type: task
title: Create dialog laid out like Jira's
status: backlog
epic: EPIC-023
parents: [STORY-0071]
estimate: 3
labels: [m10, ui]
acceptance_criteria:
  - "The top left of the dialog holds the parent stack (when one is available) and next to it the type to create as a drop-down"
  - "Title, description, priority, difficulty, tags and due date follow in Jira's order, and the dialog can be enlarged by dragging its edge"
  - "Create another and the Create button stay in the footer; the accessibility audit passes"
---

## Context
The owner asked on 2026-11-02 (during the 0.3.0 work) to make the Create dialog easier to use, closer to Jira's Create dialog, and to give new items defaults.

## Description
Redesign the Create dialog after the owner's Jira screenshots: stack picker and type drop-down in the header, title as a large field, fields below.

## Acceptance Criteria
- [ ] The top left of the dialog holds the parent stack (when one is available) and next to it the type to create as a drop-down
- [ ] Title, description, priority, difficulty, tags and due date follow in Jira's order, and the dialog can be enlarged by dragging its edge
- [ ] Create another and the Create button stay in the footer; the accessibility audit passes

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
