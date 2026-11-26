---
id: TASK-0305
type: task
title: "Exam plan, preparation window, and linked target types in studyfrog-calendar"
status: done
epic: EPIC-076
parents: ['STORY-0246']
estimate: 2
labels: [m14, calendar, core]
acceptance_criteria:
  - 'Pure data types represent ExamPlan (id, title, exam_date, window: PrepWindow, targets: Vec<ExamTarget>, created_ms, updated_ms), PrepWindow (UserWeeks(u16) | Proposed { weeks: u16, reasons: Vec<String> }), ExamTarget (Stack(String) | Item(Uid)), and ExamScheduleBlock'
  - 'Pure validation functions reject invalid exam dates or zero-week windows'
  - 'Unit tests cover construction and validation with zero inherent impl blocks and files <= 250 LOC'
---

## Context
Part of `STORY-0246`. Establishes the shared data model for the Exam Scheduler across storage, scheduling algorithms, service, and UI.

## Description
Add a pure `exam_plan` module in `studyfrog-calendar` defining `ExamPlan`, `PrepWindow`, `ExamTarget`, and `ExamScheduleBlock` plus validation free functions.

## Acceptance Criteria
- [x] Pure data types represent `ExamPlan`, `PrepWindow` (`UserWeeks` | `Proposed`), `ExamTarget` (`Stack` | `Item`), and `ExamScheduleBlock`
- [x] Pure validation functions reject invalid exam dates or zero-week windows
- [x] Unit tests cover construction and validation with zero inherent `impl` blocks and files $\le 250$ LOC

## Design Notes
- Follow ADR 0001 crate layering and Invariant P1 (zero inherent `impl` blocks).

## Dependencies
- `STORY-0246`

## Open Questions
None.
