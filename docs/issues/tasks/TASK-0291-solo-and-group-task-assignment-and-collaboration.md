---
id: TASK-0291
type: task
title: "Solo and group task assignment generator and group collaboration turns"
status: backlog
epic: EPIC-068
parents: ['STORY-0210']
estimate: 2
labels: [m14, agent, store]
acceptance_criteria:
  - 'The teacher can generate a structured task (prompt, rubric points, assignment mode) assigned either individually or to groups containing the user and simulated students'
  - 'Group tasks support simulated teammate contribution turns inside the group thread prior to submission'
  - 'Unit and service tests with a fake model verify solo and group assignment persistence and peer collaboration turns'
---

## Context
Part of `STORY-0210`. The classroom simulation must allow the teacher to assign both solo exercises and group tasks where simulated peers actively collaborate with the user.

## Description
Implement task generation and group collaboration orchestration in `studyfrog-agent` and `studyfrog-service`:
- Teacher prompt and JSON parser for structured assignments (title, prompt, grading criteria, solo vs group mode).
- Group partitioning helper assigning the user and simulated students into solo or multi-member groups.
- Simulated peer contribution turns inside a group's task workspace before turn-in.

## Acceptance Criteria
- [ ] The teacher can generate a structured task (prompt, rubric points, assignment mode) assigned either individually or to groups containing the user and simulated students
- [ ] Group tasks support simulated teammate contribution turns inside the group thread prior to submission
- [ ] Unit and service tests with a fake model verify solo and group assignment persistence and peer collaboration turns

## Design Notes
- Reuse existing JSON span extraction in `studyfrog-agent` and provide a deterministic template fallback when model output fails to parse.

## Dependencies
- `STORY-0210`, `TASK-0288`, `TASK-0290`

## Open Questions
None.
