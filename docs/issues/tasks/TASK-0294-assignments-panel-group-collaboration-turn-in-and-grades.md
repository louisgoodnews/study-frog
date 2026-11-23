---
id: TASK-0294
type: task
title: "Assignments panel, group collaboration drawer, turn-in form, and grade view"
status: backlog
epic: EPIC-068
parents: ['STORY-0211']
estimate: 3
labels: [m14, ui]
acceptance_criteria:
  - 'An assignments panel lists active and completed solo and group tasks with status badges (assigned, submitted, graded) and assignee/group rosters'
  - 'For group tasks, users can exchange messages with simulated groupmates and draft the shared submission before clicking Turn in'
  - 'Graded tasks display the teacher''s score, grade, rubric breakdown, and narrative feedback clearly in the UI'
---

## Context
Part of `STORY-0211`. Users need a clear workspace beside the lecture chat to view assigned tasks, collaborate with simulated groupmates, submit answers, and inspect teacher grades.

## Description
Build the assignment and grading UI components inside the Classroom screen (`studyfrog-ui`):
- Assignment cards showing solo vs group badge, participants, and status.
- Task detail drawer with group collaboration thread (for group tasks), answer editor, and confirmed Turn In button.
- Graded result view showing score, per-criterion feedback, and teacher comments.

## Acceptance Criteria
- [ ] An assignments panel lists active and completed solo and group tasks with status badges (`assigned`, `submitted`, `graded`) and assignee/group rosters
- [ ] For group tasks, users can exchange messages with simulated groupmates and draft the shared submission before clicking Turn in
- [ ] Graded tasks display the teacher's score, grade, rubric breakdown, and narrative feedback clearly in the UI

## Design Notes
- Ensure all buttons and inputs have clear accessible labels so `ui tree` and keyboard navigation work cleanly.

## Dependencies
- `STORY-0211`, `TASK-0292`, `TASK-0293`

## Open Questions
None.
