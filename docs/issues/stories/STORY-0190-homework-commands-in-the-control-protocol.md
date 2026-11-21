---
id: STORY-0190
type: story
title: "Homework commands in the control protocol"
status: review
epic: EPIC-064
parents: ['EPIC-064']
estimate: 2
labels: [m14, tooling, ui]
acceptance_criteria:
  - 'homework list answers with the items as the app last read them, with urgency, countdown and checklist progress'
  - 'homework new with a title and the flags due, time, estimate, priority, stack and checklist fills in the dialog and saves it; a refusal is an error with the reason'
  - 'homework advance and homework tick work, and homework tab selects a tab; list and tab are reads, the others are writes'
---

## Context
The homework screen, its tabs, the dialog and checklist ticks (STORY-0167) can only be used with the mouse. Agents and scripts need to list, create and move assignments.

## Description
Done in `studyfrog-ui`: `control_words/homework.rs` (words), `control_plan/homework.rs` (the clicks), `control_reply/homework.rs` (the answers and refusals), with the policy classes and `help` rows. Tests: `tests/control_homework.rs`.

## Acceptance Criteria
- [ ] homework list answers with the items as the app last read them, with urgency, countdown and checklist progress
- [ ] homework new with a title and the flags due, time, estimate, priority, stack and checklist fills in the dialog and saves it; a refusal is an error with the reason
- [ ] homework advance and homework tick work, and homework tab selects a tab; list and tab are reads, the others are writes

## Design Notes
The tab and the list answer from the last read; a list taken right after a change may not show it until the app has read it again.

## Dependencies
EPIC-064, and the features named above.

## Open Questions
None.
