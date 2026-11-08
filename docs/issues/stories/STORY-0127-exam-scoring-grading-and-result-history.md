---
id: STORY-0127
type: story
title: Exam scoring, grading and result history
status: done
epic: EPIC-033
parents: [EPIC-033]
estimate: 5
labels: [m12, service, slm]
acceptance_criteria:
  - "Every exam answer is scored with the semantic comparison engine (cards against back; questions against reference, accepted and rubric) and the score is stored with the run"
  - "Finishing an exam computes the percentage over all exam items (unanswered and unscored items count 0), its grade and pass or fail, stores the exam result and returns it; tested against fixture vaults with the built-in word-matching runtime"
  - "The run history used by the Runs report carries the exam result of each exam run"
---
## Context
The result of an exam must be reproducible from what is stored, not only from what the window kept.

## Description
Scoring against fixtures (TASK-0236); grading, storing and history (TASK-0237).

## Acceptance Criteria
- [x] Every exam answer is scored with the semantic comparison engine (cards against back; questions against reference, accepted and rubric) and the score is stored with the run
- [x] Finishing an exam computes the percentage over all exam items (unanswered and unscored items count 0), its grade and pass or fail, stores the exam result and returns it; tested against fixture vaults with the built-in word-matching runtime
- [x] The run history used by the Runs report carries the exam result of each exam run

## Design Notes
`studyfrog-service/src/exam.rs`; `run_history.rs` gains the result.

## Dependencies
STORY-0126.

## Open Questions
None.
