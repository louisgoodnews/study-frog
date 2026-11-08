---
id: TASK-0239
type: task
title: Exam result screen and review
status: done
epic: EPIC-033
parents: [STORY-0128]
estimate: 3
labels: [m12, ui]
acceptance_criteria:
  - "After the last answer the screen shows Grading, then the percentage, grade badge, pass or fail and one card per item with answer, reference and score; tested in the harness"
---
## Context
Part of STORY-0128.

## Description
`view/exam_result.rs`, `exam_ui.rs`.

## Acceptance Criteria
- [x] After the last answer the screen shows Grading, then the percentage, grade badge, pass or fail and one card per item with answer, reference and score; tested in the harness

## Design Notes
Scores arrive asynchronously; grading starts once every submitted answer has a score or failed to get one.

## Dependencies
TASK-0238.

## Open Questions
None.
