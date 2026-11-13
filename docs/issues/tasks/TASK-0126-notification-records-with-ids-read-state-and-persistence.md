---
id: TASK-0126
type: task
title: "Notification records with ids, read state and persistence"
status: done
epic: EPIC-023
parents: [STORY-0055]
estimate: 3
labels: [m10, ui, store]
acceptance_criteria:
  - "A notification has an id, kind, text, time, read flag and optional target"
  - "Notifications are stored in the app database, capped by count and age, and reloaded at start"
  - "Unread count is a pure function of the list"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Replace the in-memory notice list with persisted notification records and the update functions on them (add, mark, delete, clear).

## Acceptance Criteria
- [x] A notification has an id, kind, text, time, read flag and optional target
- [x] Notifications are stored in the app database, capped by count and age, and reloaded at start
- [x] Unread count is a pure function of the list

## Design Notes
`notify.rs` (pure) holds `Notification` (id, kind, text, time, read flag, optional target), the filters, the grouping by day and the update logic; `studyfrog-store::notifications` and migration 15 store them per vault (so they are in backups), capped at 200 and 30 days. The time is `state.wall_ms`, read from the system clock once per frame. Nothing is stored before the vault's list has been read; what was raised meanwhile is merged and stored then. Leaving a vault clears the list on screen and the next vault's list is read. Opening the panel no longer marks anything read (the old 'seen' count is gone): reading is the user's to do.

## Dependencies
None.

## Open Questions
None.
