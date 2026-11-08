---
id: TASK-0236
type: task
title: Score exam answers against fixtures
status: done
epic: EPIC-033
parents: [STORY-0127]
estimate: 2
labels: [m12, service, slm]
acceptance_criteria:
  - "Fixture vault tests score card answers against back and question answers against reference, accepted synonyms and rubric with the word-matching runtime, and the scores land in the run's answer scores"
---
## Context
Part of STORY-0127.

## Description
Reuses `compare::score_item_answer` and `record_answer_score`; `tests/exam.rs` with its fixtures.

## Acceptance Criteria
- [x] Fixture vault tests score card answers against back and question answers against reference, accepted synonyms and rubric with the word-matching runtime, and the scores land in the run's answer scores

## Design Notes
No new comparison code: the exam uses exactly the scoring of typed answers.

## Dependencies
TASK-0234.

## Open Questions
None.
