---
id: STORY-0042
type: story
title: "Item page like an issue view"
status: done
epic: EPIC-021
parents: [EPIC-021]
estimate: 8
labels: [m8, ui]
acceptance_criteria:
  - "One page shows and edits a stack, card, note or question with its answers, tags, links, description and fields in a side panel"
---

## Context
The owner reviewed the app again and asked for a knowledge workspace: an item page, images, subjects, reports, an AI agent and more.

## Description
Item page like an issue view.

## Acceptance Criteria
- [x] One page shows and edits a stack, card, note or question with its answers, tags, links, description and fields in a side panel

## Design Notes
The item page is view/editor.rs with view/item_details.rs; stacks edit their description and body on the same page. Replaces the editor screen; Markdown everywhere.

## Dependencies
None.

## Open Questions
None.
