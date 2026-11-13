---
id: TASK-0234
type: task
title: "Exam mode, grade scale and exam result records"
status: done
epic: EPIC-033
parents: [STORY-0126]
estimate: 3
labels: [m12, core, store]
acceptance_criteria:
  - "RunMode::Exam with run_mode_name exam and its inverse, the nine grades with letter, German grade and words, and exam_grade(percent) tested at every band boundary"
  - "Migration 25 widens the run table's mode check to exam (rows kept) and adds the exam_result table; insert, load and list are tested"
---
## Context
Part of STORY-0126.

## Description
`studyfrog-core/src/exam.rs` (grade scale, percent aggregation, exam plan), `studyfrog-store/src/exam_results.rs`, `schema_exam.rs`.

## Acceptance Criteria
- [x] RunMode::Exam with run_mode_name exam and its inverse, the nine grades with letter, German grade and words, and exam_grade(percent) tested at every band boundary
- [x] Migration 25 widens the run table's mode check to exam (rows kept) and adds the exam_result table; insert, load and list are tested

## Design Notes
Percentages are compared in tenths so floating error never moves a grade.

## Dependencies
STORY-0125.

## Open Questions
None.
