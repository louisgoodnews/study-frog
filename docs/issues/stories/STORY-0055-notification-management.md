---
id: STORY-0055
type: story
title: "Notification management"
status: done
epic: EPIC-023
parents: [EPIC-023]
estimate: 5
labels: [m10, ui, app]
acceptance_criteria:
  - "Every notification has an id and a read state that survive a restart"
  - "The notification center can mark one or all notifications read or unread, delete one, clear all and filter by unread or by errors"
  - "The bell shows the number of unread notifications"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Turn the bell's short list of recent messages into a notification center that the user can manage: read state, deletion, filtering and persistence.

## Acceptance Criteria
- [x] Every notification has an id and a read state that survive a restart
- [x] The notification center can mark one or all notifications read or unread, delete one, clear all and filter by unread or by errors
- [x] The bell shows the number of unread notifications

## Design Notes
Today the bell keeps the last notices in memory (`chrome.rs`, `track_notice`) and shows them as plain text. Notifications become data with an id, kind, text, time, read flag and optional target (the item or screen the message is about, so a click opens it). Persist in the app database so they survive a restart; cap the stored count and age by a setting. Keep the panel keyboard-operable and covered by the accessibility audit.

Tasks: TASK-0126, TASK-0127.

## Dependencies
None.

## Open Questions
None.
