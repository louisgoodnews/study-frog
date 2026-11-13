---
id: TASK-0127
type: task
title: "Notification center panel"
status: done
epic: EPIC-023
parents: [STORY-0055]
estimate: 5
labels: [m10, ui]
acceptance_criteria:
  - "Mark read and unread for one and for all, delete one, clear all"
  - "Filter by unread and by errors; group by day"
  - "A notification with a target opens it when clicked"
  - "The bell badge shows the unread count and the panel passes the accessibility audit"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Build the panel on top of the records: actions per row, bulk actions in the header, filters and the unread badge.

## Acceptance Criteria
- [x] Mark read and unread for one and for all, delete one, clear all
- [x] Filter by unread and by errors; group by day
- [x] A notification with a target opens it when clicked
- [x] The bell badge shows the unread count and the panel passes the accessibility audit

## Design Notes
`view/notifications.rs`: filter chips (All, Unread, Errors), 'Mark all read' and 'Clear all', the list grouped under Today, Yesterday and the date, each row with its time, 'Mark read' or 'Mark unread' and 'Delete' (named with the message for screen readers); a notification with a target is a link that opens it, marks it read and closes the panel. Reminders carry the calendar as their target. Not seen in a real window; tested headless with the accessibility audit.

## Dependencies
The records task of this story.

## Open Questions
None.
