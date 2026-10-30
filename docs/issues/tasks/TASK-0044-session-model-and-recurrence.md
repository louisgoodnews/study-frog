---
id: TASK-0044
type: task
title: Session model and recurrence
status: done
epic: EPIC-010
parents: [STORY-0020]
estimate: 5
labels: [m5, calendar, core]
acceptance_criteria:
  - "Civil dates (weekday, month grid, day arithmetic) shared by core, md and calendar"
  - "`Session` data with recurrence (none, daily, weekly weekdays, monthly), optional end date or count, cancelled dates"
  - "`occurrences(session, from, to)` is pure, sorted, bounded and property-tested"
  - "Monthly on the 31st skips months without one"
---

## Context
Hinnant civil-date algorithms, already used by the RFC 3339 codec.

## Description
Session model and recurrence.

## Acceptance Criteria
- [x] Civil dates (weekday, month grid, day arithmetic) shared by core, md and calendar
- [x] `Session` data with recurrence (none, daily, weekly weekdays, monthly), optional end date or count, cancelled dates
- [x] `occurrences(session, from, to)` is pure, sorted, bounded and property-tested
- [x] Monthly on the 31st skips months without one

## Design Notes
Civil-date arithmetic now lives in `core::civil` (Date, Weekday, month grid, add_days/add_months, local<->UTC with a caller-supplied offset, ISO date/time text) and `md`'s RFC 3339 codec uses it (one copy). `studyfrog-calendar`: `Session` (repeat never/daily/weekly on weekdays/monthly, ends forever/on/after N, cancelled dates), `occurrences` and `occurrences_all`, `validate_session`. Expansion is checked against a naive day-by-day oracle on 400 random cases, plus the 31st-of-the-month, every-other-week and far-future cases. `After(n)` is capped at 10,000.

## Dependencies
None.

## Open Questions
None.
