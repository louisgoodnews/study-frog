---
id: STORY-0161
type: story
title: "Markdown Inline Card Syntax Scanner"
status: backlog
epic: EPIC-054
parents: ['EPIC-054']
estimate: 5
labels: [m13, md]
acceptance_criteria:
  - 'Scanner identifies Term :: Definition delimiter syntax within standard markdown notes'
  - 'Virtual cards are generated and indexed into SQLite with source line number coordinates'
  - 'Editing the source markdown note automatically updates or deletes the virtual cards'
---

## Context
Frictionless card capture directly while authoring notes without modal dialogs.

## Description
Parse inline Term :: Definition cards from markdown notes, indexing them into SQLite review queues while preserving clean note readability.

## Acceptance Criteria
- [ ] Scanner identifies Term :: Definition delimiter syntax within standard markdown notes
- [ ] Virtual cards are generated and indexed into SQLite with source line number coordinates
- [ ] Editing the source markdown note automatically updates or deletes the virtual cards

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-054

## Open Questions
None.
