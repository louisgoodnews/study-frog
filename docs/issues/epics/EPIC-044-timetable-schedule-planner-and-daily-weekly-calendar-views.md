---
id: EPIC-044
type: epic
title: Timetable schedule planner and daily/weekly calendar views
status: backlog
parents: []
labels: [m12, calendar, ui]
acceptance_criteria:
  - "StudyFrog provides a timetable schedule planner to schedule recurring lessons, lectures, appointments, and study blocks"
  - "The calendar supports interactive switching between Month, Week, and Day views with hourly schedule agendas"
  - "Planned timetable events seamlessly integrate with review plans and rehearsal due queues"
---

## Context
Learners use StudyFrog not only for active recall flashcards, but as their central academic learning companion. Currently, the calendar is limited to a month-level grid without daily or weekly time-slot visualization, and there is no dedicated timetable planner for semester courses, lectures, or weekly lesson schedules.

## Description
Deliver an academic timetable and multi-scale calendar view system:
1. **Timetable Schedule Planner (`STORY-0141`):** Schedule weekly courses, recurring lectures, seminars, lab sessions, and study slots with room, instructor, and subject metadata.
2. **Daily and Weekly Calendar Views (`STORY-0142`):** Provide interactive Week and Day views featuring hourly time slots, event overlap handling, and unified display of lectures alongside planned SRS reviews.

## Acceptance Criteria
- [ ] StudyFrog provides a timetable schedule planner to schedule recurring lessons, lectures, appointments, and study blocks
- [ ] The calendar supports interactive switching between Month, Week, and Day views with hourly schedule agendas
- [ ] Planned timetable events seamlessly integrate with review plans and rehearsal due queues

## Design Notes
- Touchpoints: `crates/studyfrog-calendar`, `crates/studyfrog-store`, `crates/studyfrog-ui/src/view/calendar.rs`, `calendar_grid.rs`.
- Child stories: STORY-0141, STORY-0142.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-023 (Calendar and review planning).

## Open Questions
- None.
