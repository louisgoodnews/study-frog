---
id: TASK-0162
type: task
title: User-added and user-edited help pages
status: backlog
epic: EPIC-023
parents: [STORY-0066]
estimate: 5
labels: [m10, ui, md, store]
acceptance_criteria:
  - "A user can create a page, edit it, rename it and delete it, and it is stored as Markdown under wiki/ in the vault"
  - "Supplied pages are read-only but can be copied into a user page"
  - "User pages appear in the page tree and the help search, and are included in backups"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add the wiki folder to the vault scanner and the editing to the help screen.

## Acceptance Criteria
- [ ] A user can create a page, edit it, rename it and delete it, and it is stored as Markdown under wiki/ in the vault
- [ ] Supplied pages are read-only but can be copied into a user page
- [ ] User pages appear in the page tree and the help search, and are included in backups

## Design Notes
None.

## Dependencies
The viewer task of this story.

## Open Questions
None.
