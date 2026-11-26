---
id: TASK-0310
type: task
title: "Surface exam-scheduled blocks in Calendar, Home, and one-click scoped Rehearsal"
status: backlog
epic: EPIC-076
parents: ['STORY-0248']
estimate: 2
labels: [m14, calendar, ui]
acceptance_criteria:
  - 'Scheduled exam study blocks appear in Calendar (Month, Week, Day views) and Home ("Coming up" / "Due now") with the exam title and item count'
  - 'Clicking a scheduled exam block offers a one-click "Rehearse exam items" action that starts a rehearsal run scoped to that day''s scheduled item set'
  - 'Completing scheduled reviews updates remaining exam block progress cleanly'
---

## Context
Part of `STORY-0248`. Makes generated exam schedules actionable right from the daily views users already visit.

## Description
Integrate persisted `ExamScheduleBlock` rows into Calendar views, Home summaries, and Rehearsal launch in `studyfrog-service` and `studyfrog-ui`.

## Acceptance Criteria
- [ ] Scheduled exam study blocks appear in Calendar (Month, Week, Day views) and Home (`Coming up` / `Due now`) with the exam title and item count
- [ ] Clicking a scheduled exam block offers a one-click `"Rehearse exam items"` action that starts a rehearsal run scoped to that day's scheduled item set
- [ ] Completing scheduled reviews updates remaining exam block progress cleanly

## Design Notes
- Reuse existing `studyfrog-calendar` entry rendering and `Msg::Home` / `Msg::Rehearse` launch paths.

## Dependencies
- `STORY-0248`, `TASK-0309`

## Open Questions
None.
