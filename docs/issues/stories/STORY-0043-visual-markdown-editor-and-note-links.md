---
id: STORY-0043
type: story
title: Visual Markdown editor and note links
status: done
epic: EPIC-021
parents: [EPIC-021]
estimate: 8
labels: [m8, ui]
acceptance_criteria:
  - "New notes open a full visual Markdown editor"
  - "Cards and questions can link to notes and the reverse, with backlinks"
---

## Context
The owner reviewed the app again and asked for a knowledge workspace: an item page, images, subjects, reports, an AI agent and more.

## Description
Visual Markdown editor and note links.

## Acceptance Criteria
- [x] New notes open a full visual Markdown editor
- [x] Cards and questions can link to notes and the reverse, with backlinks

## Design Notes
view/md_editor.rs (toolbar, Write/Split/Preview) and md_edit.rs (pure commands); the Create dialog uses it for every text area and widens for notes; backlinks via store::linking_to. Depends on STORY-0042.

## Dependencies
None.

## Open Questions
None.
