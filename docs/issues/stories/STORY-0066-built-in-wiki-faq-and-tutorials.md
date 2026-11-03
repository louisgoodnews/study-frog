---
id: STORY-0066
type: story
title: Built-in wiki, FAQ and tutorials
status: done
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, ui, docs]
acceptance_criteria:
  - "A help area inside the app holds the vision behind StudyFrog, how-to pages, a FAQ and tutorials in English, readable offline and read-only"
  - "The user can add and edit their own help pages, kept in the vault"
  - "Help is searchable and every screen links to its page from a help button, and a first-run tutorial walks a new user through a stack, a card, a rehearsal and the calendar"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Let a user learn what StudyFrog is for and how to use it without leaving the app, and keep their own notes about using it.

## Acceptance Criteria
- [x] A help area inside the app holds the vision behind StudyFrog, how-to pages, a FAQ and tutorials in English, readable offline and read-only
- [x] The user can add and edit their own help pages, kept in the vault
- [x] Help is searchable and every screen links to its page from a help button, and a first-run tutorial walks a new user through a stack, a card, a rehearsal and the calendar

## Design Notes
The supplied content is Markdown in the repository (`docs/help/`), compiled into the binary so it works offline and is versioned with the code, and shown by the existing Markdown preview. It is English and read-only (the owner's decision, 2026-11-02). Pages: Why StudyFrog (from `docs/VISION.md`, written for users), Getting started, one how-to per screen, the FAQ, a glossary (from `docs/GLOSSARY.md`) and a page on privacy and local data. The user may add pages and edit the pages they added, in the same Markdown editor as notes; they live in a `wiki/` folder of the vault so they are backed up, searched and linked like notes. Supplied pages cannot be changed, but a user page can link to them and can be created as a copy of one. Tutorials are short step lists that open the screen they describe and can be repeated; the first-run tutorial is offered once and can be skipped. The search covers supplied and user pages with the same search code as the vault. The `?` in the top bar opens help; a help button on each screen opens the matching page. A check script fails when a supplied page links to a page that does not exist.

Tasks: TASK-0161, TASK-0162, TASK-0163, TASK-0164, TASK-0165, TASK-0166.

## Dependencies
Content for features of 0.3.0 can be written as each feature lands; the viewer stands alone.

## Open Questions
Answered by the owner on 2026-11-02: English and read-only for the supplied pages, plus user-added and user-edited pages. The assistant's proposals for the vault (`view/agent_wiki.rs`, `studyfrog_agent::wiki::Change`) are linked to the help wiki: confirmed by the owner on 2026-11-02, the assistant may read the user's pages and propose new pages and edits that the user must confirm, like its other proposals. It never changes a supplied page.
