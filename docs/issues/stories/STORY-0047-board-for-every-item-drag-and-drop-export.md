---
id: STORY-0047
type: story
title: Board for every item, drag and drop, export
status: done
epic: EPIC-021
parents: [EPIC-021]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "Imports and the library card form add board cards"
  - "Cards can be dragged between columns"
---

## Context
The owner reviewed the app again and asked for a knowledge workspace: an item page, images, subjects, reports, an AI agent and more.

## Description
Board for every item, drag and drop, export.

## Acceptance Criteria
- [x] Imports and the library card form add board cards
- [x] Cards can be dragged between columns

## Design Notes
The export moved into STORY-0050 (backups): the board and the calendar live only in the database, so a backup must carry them. Dragging already worked; cards are now draggable by their whole face. No export exists yet; this adds one.

## Dependencies
None.

## Open Questions
None.
