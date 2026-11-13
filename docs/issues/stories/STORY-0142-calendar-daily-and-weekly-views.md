---
id: STORY-0142
type: story
title: "Calendar daily and weekly schedule views"
status: done
epic: EPIC-044
parents: [EPIC-044]
estimate: 5
labels: [m12, calendar, ui]
acceptance_criteria:
  - "The calendar interface provides segmented controls to switch between Month, Week, and Day views"
  - "The Week view displays 7 daily columns with an hourly time grid from 06:00 to 22:00 showing appointments, lectures, tasks, and reviews"
  - "The Day view displays a detailed single-day agenda with time blocks, stack links, and quick event creation"
---

## Context
While the month view provides high-level review density and deadlines, daily time management requires visualizing hour-by-hour commitments (lectures, seminars, study sessions). Learners requested daily and weekly views to plan their day effectively.

## Description
Implement Week and Day views on the calendar screen:
1. Add a view mode toggle (`Month | Week | Day`) in `crates/studyfrog-ui/src/view/calendar.rs`.
2. Implement `calendar_week.rs`: 7-day column grid with hour markers, rendering session blocks, timetable classes, and review projections.
3. Implement `calendar_day.rs`: Single-day hourly agenda with detailed card summaries, duration indicators, and direct links to start rehearsals for due stacks.
4. Support drag-and-drop or click-to-schedule at specific hours.

## Acceptance Criteria
- [x] The calendar interface provides segmented controls to switch between Month, Week, and Day views
- [x] The Week view displays 7 daily columns with an hourly time grid from 06:00 to 22:00 showing appointments, lectures, tasks, and reviews
- [x] The Day view displays a detailed single-day agenda with time blocks, stack links, and quick event creation

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/calendar.rs`, `crates/studyfrog-ui/src/view/calendar_week.rs`, `crates/studyfrog-ui/src/view/calendar_day.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).

## Dependencies
- EPIC-044 (Timetable schedule planner and daily/weekly calendar views).

## Open Questions
- None.
