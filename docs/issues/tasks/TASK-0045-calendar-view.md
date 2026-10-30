---
id: TASK-0045
type: task
title: Calendar view
status: backlog
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
- [ ] Month grid with previous, next and today; sessions shown on their days (recurring ones too)
- [ ] A day panel lists sessions with time and stack, and offers edit, delete and 'Start rehearsal'
- [ ] A form creates sessions (title, date, time, duration, stack, repeat)
- [ ] Headless tests including an accessibility audit

## Design Notes
Occurrence expansion runs in update when the visible month changes, never per frame.

## Dependencies
TASK-0078, TASK-0079.

## Open Questions
None.
