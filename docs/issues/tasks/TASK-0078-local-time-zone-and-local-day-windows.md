---
id: TASK-0078
type: task
title: "Local time zone and local-day windows"
status: done
epic: EPIC-010
parents: [STORY-0020]
estimate: 3
labels: [m5, service, core]
acceptance_criteria:
  - "An offset provider turns UTC instants into local civil date-times and back"
  - "Daily limits and the 'today' of queue building use the local day, not the UTC day"
  - "The provider is the only place that reads the system time zone"
---

## Context
Fixes the 'UTC day' limitation flagged in STATUS.md.

## Description
Local time zone and local-day windows.

## Acceptance Criteria
- [x] An offset provider turns UTC instants into local civil date-times and back
- [x] Daily limits and the 'today' of queue building use the local day, not the UTC day
- [x] The provider is the only place that reads the system time zone

## Design Notes
`studyfrog-service::timezone`: `local_offset_minutes` (chrono, clock feature, the only chrono use), `day_window_at(now, offset_fn)` (pure, handles clock changes by refining the offset at midnight) and `local_day_window`. Run previews, run creation and daily limits now use the local day. Tested with a simulated spring-forward (a 23-hour day, consecutive days tile).

## Dependencies
TASK-0044.

## Open Questions
A midnight that does not exist resolves to the first instant after the gap; an ambiguous one to the earlier instant.
