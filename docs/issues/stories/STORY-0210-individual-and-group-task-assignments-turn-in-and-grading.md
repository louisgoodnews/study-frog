---
id: STORY-0210
type: story
title: "Individual and group task assignments, turn-in workflow, and automated grading"
status: backlog
epic: EPIC-068
parents: ['EPIC-068']
estimate: 5
labels: [m14, agent, store]
acceptance_criteria:
  - 'The simulated teacher can issue structured tasks assigned either individually or to groups containing the user and/or simulated students'
  - 'In group tasks, simulated teammates contribute drafts or discussion turns before the group submission is finalized'
  - 'Assigned tasks have an explicit turn-in step that locks the submission and triggers teacher grading'
  - 'Teacher grading returns a numeric/letter score and structured criterion feedback persisted with the session, with a deterministic fallback if the model is unavailable'
---

## Context
Listening to a lecture is only half of a classroom experience; active practice through teacher-assigned individual and group tasks, explicit submission, and graded feedback cements learning and reveals gaps.

## Description
Build the assignment, collaboration, submission, and grading engine in `studyfrog-agent`, `studyfrog-service`, and `studyfrog-store`:
- Teacher-generated tasks tied to the current lecture segment (prompt, criteria/rubric, solo vs group mode, assignee roster).
- Group collaboration loop where simulated groupmates propose ideas or sub-answers and respond to the user's edits before turn-in.
- Explicit turn-in transition recording final individual or group submissions (including simulated peers' submissions for comparison).
- Grading pipeline evaluating submissions against the task rubric, returning scores and actionable feedback saved to local storage.

## Acceptance Criteria
- [ ] The simulated teacher can issue structured tasks assigned either individually or to groups containing the user and/or simulated students
- [ ] In group tasks, simulated teammates contribute drafts or discussion turns before the group submission is finalized
- [ ] Assigned tasks have an explicit turn-in step that locks the submission and triggers teacher grading
- [ ] Teacher grading returns a numeric/letter score and structured criterion feedback persisted with the session, with a deterministic fallback if the model is unavailable

## Design Notes
- Reuse rubric-evaluation and JSON-extraction helpers from `studyfrog-agent` (`rubric_eval`, `exam_grade`) where applicable.
- Keep submission and grading state transitions pure in `studyfrog-core` and persisted atomically in `studyfrog-store`.

## Dependencies
- `EPIC-068`, `STORY-0208`, `STORY-0209`
- Child tasks: `TASK-0291`, `TASK-0292`

## Open Questions
- Whether group grades apply uniformly to all group members or also note individual contributions.
