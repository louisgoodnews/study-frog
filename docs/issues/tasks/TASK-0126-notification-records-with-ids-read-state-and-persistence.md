---
id: TASK-0126
type: task
title: Notification records with ids, read state and persistence
status: backlog
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
- [ ] A notification has an id, kind, text, time, read flag and optional target
- [ ] Notifications are stored in the app database, capped by count and age, and reloaded at start
- [ ] Unread count is a pure function of the list

## Design Notes
None.

## Dependencies
None.

## Open Questions
None.
