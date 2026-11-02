---
id: STORY-0071
type: story
title: Create dialog like Jira
status: done
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, ui]
acceptance_criteria:
  - "A new item without a priority or difficulty gets medium and difficult"
  - "The Create dialog is laid out like Jira's: parent stack and type drop-down at the top left, then the fields"
  - "Multi-line fields are Markdown editors that can be enlarged by dragging"
---

## Context
The owner asked on 2026-11-02 (during the 0.3.0 work) to make the Create dialog easier to use, closer to Jira's Create dialog, and to give new items defaults.

## Description
The Create dialog is hard to use. Redesign it after the owner's Jira screenshots and give new items sensible defaults.

## Acceptance Criteria
- [x] A new item without a priority or difficulty gets medium and difficult
- [x] The Create dialog is laid out like Jira's: parent stack and type drop-down at the top left, then the fields
- [x] Multi-line fields are Markdown editors that can be enlarged by dragging

## Design Notes
Header: the stack the item goes into (a drop-down listing the stacks, when there is one) and, next to it, the type (Card, Note, Question, Stack) as a drop-down, with minimise-free window buttons: enlarge and close. Below: the title as a large borderless field, then the description editor, then the rest as labelled fields. The dialog and the editors grow when dragged by their lower right corner. Footer: Create another and the primary Create button.

Tasks: TASK-0185, TASK-0186, TASK-0187.

## Dependencies
None.

## Open Questions
None.
