---
id: TASK-0136
type: task
title: "Record how long items are viewed and acted on"
status: done
epic: EPIC-023
parents: [STORY-0058]
estimate: 3
labels: [m10, store, ui]
acceptance_criteria:
  - "Opening and closing an item page records a view with its duration, and idle time is not counted"
  - "Editing and answering record action time"
  - "Stored events are included in backups and can be switched off in Settings"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Add view and action events to the store with a migration, and emit them from the item page and the rehearsal screen.

## Acceptance Criteria
- [x] Opening and closing an item page records a view with its duration, and idle time is not counted
- [x] Editing and answering record action time
- [x] Stored events are included in backups and can be switched off in Settings

## Design Notes
Idle time (no input for a minute) must not count. The tracking stays on the user's machine like all other data.
Done as `view_time.rs` (a pure tracker over `state.now`, input signalled by `Msg::Activity`, at most 5 s per tick so a suspended computer adds nothing, visits under 1 s dropped), table `item_view` (migration 13), `Effect::RecordView` and the `track_views` setting in `app.toml`. An `edit` event is written beside the `view` when the item was changed or saved during the visit. Answering time is not duplicated: it is derived from the timestamps of `run_event`, which the rehearsal already stores. A visit that is open when the app is closed is lost.

## Dependencies
The specification task of this story.

## Open Questions
None.
