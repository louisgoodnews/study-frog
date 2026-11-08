---
id: TASK-0242
type: task
title: Exam runs in the control protocol
status: done
epic: EPIC-033
parents: [STORY-0129]
estimate: 2
labels: [m12, ui, app]
acceptance_criteria:
  - "rehearse setup --mode exam, start, type, submit, answer and finish work over the control protocol; state shows the exam (answered, locked answers, grading, result); flip, skip, rate, mark and blank answers are refused with an error; tested headless"
---
## Context
Part of STORY-0129.

## Description
`control_words.rs`, `control_plan.rs`, `control_parts.rs`, `control_help.rs`, `tests/control_exam.rs`.

## Acceptance Criteria
- [x] rehearse setup --mode exam, start, type, submit, answer and finish work over the control protocol; state shows the exam (answered, locked answers, grading, result); flip, skip, rate, mark and blank answers are refused with an error; tested headless

## Design Notes
None.

## Dependencies
STORY-0124.

## Open Questions
None.
