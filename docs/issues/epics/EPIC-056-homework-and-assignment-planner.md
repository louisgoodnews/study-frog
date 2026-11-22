---
id: EPIC-056
type: epic
title: "Homework and Assignment Planner: Due Dates, Reminders, and Timetable Integration"
status: review
parents: []
labels: [m14, calendar, ui]
acceptance_criteria:
  - 'Dedicated homework domain model supporting title, subject/course, due date/time, estimated effort, priority, and completion status'
  - 'Proactive notification and reminder triggers alerting students before due dates (e.g. 24h, 2h, 15m lead times)'
  - 'Integration with timetable lectures and calendar views displaying upcoming assignment deadlines'
  - 'Dedicated Homework planner UI with filtering by subject, urgency status (overdue, due today, upcoming), and completion toggles'
---

## Context
Students balancing coursework across multiple subjects need an integrated system to track assignments, problem sets, readings, and project deadlines directly alongside their class timetable, flashcard rehearsal, and study notes.

## Description
Deliver an end-to-end Homework & Assignment Management subsystem within StudyFrog:
1. **Homework Model & State Machine**: Data structures for assignments with course linkage, due timestamp, estimated effort in minutes, priority levels, and completion state.
2. **Proactive Lead-Time Reminders**: Integration into StudyFrog's pure reminder scheduling engine to fire notifications before deadlines (e.g. 1 day, 2 hours, or 15 minutes before due date).
3. **Calendar & Timetable Synergy**: Display assignment deadlines on daily and weekly calendar views, correlating homework with specific lectures or seminars.
4. **Dedicated Planner UI**: A clean, accessible view with quick task capture, status toggling, subject filtering, and visual urgency badges (overdue, due today, upcoming).

## Acceptance Criteria
- [x] Dedicated homework domain model supporting title, subject/course, due date/time, estimated effort, priority, and completion status
- [x] Proactive notification and reminder triggers alerting students before due dates (e.g. 24h, 2h, 15m lead times)
- [ ] Integration with timetable lectures and calendar views displaying upcoming assignment deadlines
- [ ] Dedicated Homework planner UI with filtering by subject, urgency status (overdue, due today, upcoming), and completion toggles

## Design Notes
Pure domain models should live in `studyfrog-calendar` or `studyfrog-core`, extending the existing `Date`, `LocalTime`, and `Session` infrastructure without inherent `impl` blocks (Invariant P1). In-vault persistence will follow the standard local-first human-readable format.

## Dependencies
None.

## Open Questions
None.

## Implementation (2026-11-22, audit)

Audit (2026-11-22): criterion 1 MET: the homework list (`ctl homework list`) holds title, course (stack), due date and time, estimate, priority, status and checklist steps; the course chip "Statistik" shows on a card (/tmp/claude-0/audit/g5/0166-homework-mini.png). Criterion 2 MET: 24 h (Lektuere Kapitel 4, due in 11 h), 2 h (Kurzquiz) and 15 min (Mini Test, due in 18 min when the 2 h lead had already shown) leads each fired once (notices; /tmp/claude-0/audit/g5/0166-homework-all.png). Criterion 3 NOT MET as written: the calendar views show deadlines (/tmp/claude-0/audit/g5/0166-month.png, 0166-day-kurzquiz.png), but the Timetable view shows no homework (/tmp/claude-0/audit/g5/0166-timetable.png) and no lectures were available to check the correlation; left open. Criterion 4 NOT MET: the planner has urgency tabs (All, Active, Due today, Overdue, Completed) and Start, Reopen and checklist ticks (/tmp/claude-0/audit/g5/0166-homework-all.png, 0166-homework-completedtab.png), but no subject filter (/tmp/claude-0/audit/g5/0166-homework-all.png shows none).
