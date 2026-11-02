---
id: STORY-0069
type: story
title: Stack detail view like Jira's issue view
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, ui]
acceptance_criteria:
  - "A stack opens in a detail view laid out like Jira's issue view: content on the left, a details panel on the right"
  - "It opens as a side panel from the stack list, the board, the calendar and the overview without losing the user's place, and as a full page"
  - "An activity section shows the stack's reviews, edits and status changes, and the same layout serves cards, notes and questions"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Separate the stack's own detail page from the stack overview, as the owner asked on 2026-11-02: the overview is Jira's project view (STORY-0063), the detail view is Jira's issue view, with a stack as the item.

## Acceptance Criteria
- [ ] A stack opens in a detail view laid out like Jira's issue view: content on the left, a details panel on the right
- [ ] It opens as a side panel from the stack list, the board, the calendar and the overview without losing the user's place, and as a full page
- [ ] An activity section shows the stack's reviews, edits and status changes, and the same layout serves cards, notes and questions

## Design Notes
The owner confirmed on 2026-11-02 that the 'stack item view' is the stack's own detail page. In StudyFrog a stack is an item like a card or a note, so the detail view is one layout for every kind and reshapes the item page of STORY-0042. Left column: breadcrumb (parent stacks), title, the description in Markdown, then for a stack its child items (sub-stacks, cards, notes, questions) as a compact list with status and due date, its PDFs and its links; for a card, note or question the content, answers and pictures instead. Below: an Activity section with tabs All, Reviews, Edits and Status changes. Right-hand Details panel: status, priority, difficulty, familiarity, next review, due date, tags, subjects and teachers, scheduler and budgets for a stack, created and changed times, each editable in place. From the list, board and calendar a click opens the item in a side panel next to them, with a button to expand it to a full page and keys to step to the next and previous item. Edits made in the panel show in the list at once. The activity history needs a log of edits and status changes; reviews are already stored.

Tasks: TASK-0174, TASK-0175, TASK-0176, TASK-0177.

## Dependencies
STORY-0061 (statuses) and STORY-0058 (familiarity) for two details; the list (STORY-0057) for opening in a panel; STORY-0063 links to it from the overview header.

## Open Questions
Should a stack's child items be editable in the left column, or only listed? Proposal: listed with in-place status changes, and a click opens the child in the same panel.
