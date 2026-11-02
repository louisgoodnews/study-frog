---
id: STORY-0057
type: story
title: Library redesign
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, ui]
acceptance_criteria:
  - "The library is a copy of Jira's list view: a sortable table of items with columns for type, status, priority, difficulty, familiarity and due date, and a count below"
  - "A search bar toggles between Standard (filter fields) and Query (an SQL-like syntax) and both narrow the same list"
  - "Saved filters and bulk actions work on the list, and creating cards and importing live in dialogs, not in the page"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
The library screen has the stack tree, the stack page, the card form and the import form all in one view, which makes it hard to use. Redesign it as a copy of the Jira list view in the owner's screenshots, with an SQL-like search.

## Acceptance Criteria
- [ ] The library is a copy of Jira's list view: a sortable table of items with columns for type, status, priority, difficulty, familiarity and due date, and a count below
- [ ] A search bar toggles between Standard (filter fields) and Query (an SQL-like syntax) and both narrow the same list
- [ ] Saved filters and bulk actions work on the list, and creating cards and importing live in dialogs, not in the page

## Design Notes
Layout (accepted by the owner on 2026-11-02: copy Jira's list view): a collapsible stack tree on the left, and a table of items on the right with a search bar above and a count below ('50 of 1000+' style). Columns can be shown, hidden and sorted; rows can be selected for bulk actions (set status, priority, move to a stack, delete). The search bar has two modes like Jira's Standard and JQL toggle: Standard shows filter fields (text, type, status, stack, subject, due), Query takes a typed expression such as `type = card AND status IN (new, learning) AND familiarity < 0.4 AND due <= 7d ORDER BY familiarity ASC`. The syntax covers fields, comparison, IN, AND, OR, NOT, parentheses, text match with `~`, relative dates and ORDER BY. A hand-written parser (build, not buy, P10) produces a syntax tree that is evaluated against the index; values are bound as parameters, never pasted into SQL, so a query cannot reach anything but the items. Errors name the position and the expected token, and the field names complete as you type. Standard mode and Query mode convert into each other, and a saved filter stores the query. The list is virtualised so 10k items scroll within the frame budget, measured. The card form and the import form move into the Create dialog and a toolbar menu.

Tasks: TASK-0129, TASK-0130, TASK-0131, TASK-0132, TASK-0133, TASK-0134.

## Dependencies
Statuses (STORY-0061) and familiarity (STORY-0058) supply two of the columns and two query fields; the list can ship first without them. The stack view's List and Backlog tabs (STORY-0064) reuse the list and the query bar.

## Open Questions
Which columns are shown by default? Proposal: title, type, status, priority, due. Should queries also reach note text, or only fields and titles? Proposal: titles and fields with `~`, and a separate full-text term.
