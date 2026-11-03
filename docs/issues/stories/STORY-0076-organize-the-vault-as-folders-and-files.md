---
id: STORY-0076
type: story
title: Organize the vault as folders and files
status: backlog
epic: EPIC-026
parents: [EPIC-026]
estimate: 8
labels: [m11, ui, store]
acceptance_criteria:
  - "A Folders view shows the vault's folders and files and lets the user create, rename, move and delete them"
  - "An Items view shows the items as they are, as now, and the two views are told apart in the navigation"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
Offer two views of the same vault: the folders and files as they are on disk, and the items as StudyFrog understands them.

## Acceptance Criteria
- [ ] A Folders view shows the vault's folders and files and lets the user create, rename, move and delete them
- [ ] An Items view shows the items as they are, as now, and the two views are told apart in the navigation

## Design Notes
Tasks TASK-0202 to TASK-0204. Depends on STORY-0079: the owner decided on 2026-11-03 that folders are only folders and a stack is a file that points to its contents.

## Dependencies
None.

## Open Questions
None.
