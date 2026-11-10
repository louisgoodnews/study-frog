---
id: STORY-0141
type: story
title: Timetable schedule planner for lectures and lessons
status: backlog
epic: EPIC-044
parents: [EPIC-044]
estimate: 5
labels: [m12, calendar, service, ui]
acceptance_criteria:
  - "Learners can define recurring weekly timetable entries for courses, lectures, seminars, and lessons"
  - "Each timetable entry stores course title, day of week, start/end time, location/room, instructor, and optional associated stack"
  - "Timetable entries automatically project into calendar events and notifications across the semester term"
---

## Context
Students need to organize their weekly academic schedule directly in StudyFrog. Having lecture times and study periods mapped directly to related flashcard stacks allows automatic linking between classroom instruction and spaced repetition reviews.

## Description
Deliver the timetable schedule planner in `crates/studyfrog-calendar` and `crates/studyfrog-ui`:
1. Extend `studyfrog-calendar` data models and `studyfrog-store` SQLite schema with a `timetable_entry` model (`id`, `subject`, `day_of_week`, `start_time`, `end_time`, `location`, `instructor`, `stack_uid`).
2. Provide a Timetable management screen/modal allowing users to add, edit, and manage weekly recurring class schedules.
3. Automatically generate calendar occurrences for timetable classes on the calendar and notification system.

## Acceptance Criteria
- [ ] Learners can define recurring weekly timetable entries for courses, lectures, seminars, and lessons
- [ ] Each timetable entry stores course title, day of week, start/end time, location/room, instructor, and optional associated stack
- [ ] Timetable entries automatically project into calendar events and notifications across the semester term

## Design Notes
- Touchpoints: `crates/studyfrog-calendar/src/session.rs`, `crates/studyfrog-store/src/schema.rs`, `crates/studyfrog-ui/src/view/calendar.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-044 (Timetable schedule planner and daily/weekly calendar views).

## Open Questions
- None.
