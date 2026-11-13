---
id: STORY-0050
type: story
title: "Backups and restores"
status: done
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
- [x] A backup holds the vault content and the configuration, and optionally the keys
- [x] A restore brings them back after a check and a confirmation

## Design Notes
service::backup and restore write and read a plain tar (service::tar), with manifest.toml first, a VACUUM INTO copy of the database, the vault files, and optionally settings and keys. A restore goes into a new or empty folder. A zip-like archive written to a folder the user picks; the database is rebuilt from the files.

## Dependencies
None.

## Open Questions
None.
