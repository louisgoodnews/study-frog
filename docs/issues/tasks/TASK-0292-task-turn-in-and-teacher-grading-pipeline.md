---
id: TASK-0292
type: task
title: "Task turn-in workflow and teacher grading pipeline with structured feedback"
status: done
epic: EPIC-068
parents: ['STORY-0210']
estimate: 3
labels: [m14, agent, store]
acceptance_criteria:
  - 'Submitting a solo or group task locks the submission in local storage and optionally records simulated peers'' submissions'
  - 'The teacher grades submitted work against the task rubric via local or remote model, returning numeric score, grade label, per-criterion notes, and summary feedback'
  - 'A deterministic keyword/rubric fallback grades submissions if the model call fails, and integration tests verify both model and fallback paths'
---

## Context
Part of `STORY-0210`. Assigned classroom tasks need a clear turn-in action followed by reliable, persisted teacher grading and feedback.

## Description
Build submission and grading use cases in `studyfrog-agent` and `studyfrog-service`:
- Explicit turn-in function validating assignment state and persisting final answers.
- Teacher grading prompt and response parser returning structured scores and feedback per rubric criterion.
- Fallback lexical/rubric scorer so turn-in never leaves a submission ungradable if the model times out or returns malformed JSON.

## Acceptance Criteria
- [x] Submitting a solo or group task locks the submission in local storage and optionally records simulated peers' submissions
- [x] The teacher grades submitted work against the task rubric via local or remote model, returning numeric score, grade label, per-criterion notes, and summary feedback
- [x] A deterministic keyword/rubric fallback grades submissions if the model call fails, and integration tests verify both model and fallback paths

## Design Notes
- Align grading output structure with existing `studyfrog-agent::exam_grade` / `rubric_eval` patterns.

## Dependencies
- `STORY-0210`, `TASK-0291`

## Open Questions
None.
