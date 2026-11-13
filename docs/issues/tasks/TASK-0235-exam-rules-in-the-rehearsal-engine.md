---
id: TASK-0235
type: task
title: "Exam rules in the rehearsal engine"
status: done
epic: EPIC-033
parents: [STORY-0126]
estimate: 2
labels: [m12, rehearsal]
acceptance_criteria:
  - "In an exam the engine refuses flip, skip, rate and mark with a named error and refuses blank answers"
  - "A submitted answer is locked (one AnswerSubmitted per item) and the next item is shown at once; property tests cover arbitrary input sequences"
---
## Context
Part of STORY-0126.

## Description
`studyfrog-rehearsal/src/exam.rs`, new `StepError` variants, `tests/exam_rules.rs`.

## Acceptance Criteria
- [x] In an exam the engine refuses flip, skip, rate and mark with a named error and refuses blank answers
- [x] A submitted answer is locked (one AnswerSubmitted per item) and the next item is shown at once; property tests cover arbitrary input sequences

## Design Notes
No new actions: the existing `SubmitAnswer` is the only way forward.

## Dependencies
TASK-0234.

## Open Questions
None.
