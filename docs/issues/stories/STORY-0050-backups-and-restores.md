---
id: STORY-0050
type: story
title: Backups and restores
status: backlog
epic: EPIC-021
parents: [EPIC-021]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "A backup holds the vault content, the board, the calendar and the configuration, and optionally the keys"
  - "A restore brings them back after a check and a confirmation"
---

## Context
The owner asked for this after reviewing the app on 2026-10-30.

## Description
Backups and restores.

## Acceptance Criteria
- [ ] A backup holds the vault content and the configuration, and optionally the keys
- [ ] A restore brings them back after a check and a confirmation

## Design Notes
A zip-like archive written to a folder the user picks; the database is rebuilt from the files.

## Dependencies
None.

## Open Questions
None.
