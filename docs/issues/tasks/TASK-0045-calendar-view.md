---
id: TASK-0045
type: task
title: "Calendar view"
status: done
epic: EPIC-010
parents: [STORY-0020]
estimate: 5
labels: [m5, ui, calendar]
acceptance_criteria:
  - "Month grid with previous, next and today; sessions shown on their days (recurring ones too)"
  - "A day panel lists sessions with time and stack, and offers edit, delete and 'Start rehearsal'"
  - "A form creates sessions (title, date, time, duration, stack, repeat)"
  - "Headless tests including an accessibility audit"
---

## Context
Planning a week of study should take seconds.

## Description
Calendar view.

## Acceptance Criteria
- [x] Month grid with previous, next and today; sessions shown on their days (recurring ones too)
- [x] A day panel lists sessions with time and stack, and offers edit, delete and 'Start rehearsal'
- [x] A form creates sessions (title, date, time, duration, stack, repeat)
- [x] Headless tests including an accessibility audit

## Design Notes
Occurrence expansion runs in update when the visible month changes, never per frame.

## Dependencies
TASK-0078, TASK-0079.

## Open Questions
None.

## Outcome
Calendar screen (`planner`, `session_form`, `view/calendar`, `view/session_form`): month grid with previous/next/today and a per-day count that includes recurring occurrences (expanded in `update` when the month or sessions change, never per frame); a day panel with edit, delete, skip-this-day and "Start rehearsal" (opens the rehearse screen filtered to the session's stack, spaced preset); a form with title, date, time, duration, stack, repeat (daily/weekly with weekdays/monthly), end rule and notes. Invalid input keeps the form open with the reason. Covered by `planning_flow` (real executor and vault) and `planning_views` (headless, accessibility audit).
