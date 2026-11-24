---
id: TASK-0291
type: task
title: "Solo and group task assignment generator and group collaboration turns"
status: done
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
- [x] The teacher can generate a structured task (prompt, rubric points, assignment mode) assigned either individually or to groups containing the user and simulated students
- [x] Group tasks support simulated teammate contribution turns inside the group thread prior to submission
- [x] Unit and service tests with a fake model verify solo and group assignment persistence and peer collaboration turns

## Design Notes
- Reuse existing JSON span extraction in `studyfrog-agent` and provide a deterministic template fallback when model output fails to parse.
- Build vs buy: the group partitioning, task parser, fallback template and teammate prompts are written in `studyfrog-agent`; the reply is read with `serde_json`, already a dependency. No new crate.
- Shape change: `Assignment` gains `rubric` (grading needs the points persisted), stored in migration 33 (`classroom_rubric`). The group thread is the session's chat turns by the group's members, since the core keeps one chat per session; it is not a separate per-group log.

## Dependencies
- `STORY-0210`, `TASK-0288`, `TASK-0290`

## Open Questions
None.
