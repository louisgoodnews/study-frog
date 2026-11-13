---
id: TASK-0054
type: task
title: "Answer input in rehearsal"
status: done
epic: EPIC-013
parents: [STORY-0025]
estimate: 3
labels: [m6, ui]
acceptance_criteria:
  - "Submitting a typed answer asks for a comparison; the result shows the match, the verdict and the suggested rating"
  - "The user still rates by hand; a failed or late comparison never blocks rating"
  - "The service compares against a card (back) or a question (reference, accepted, rubric, min_score)"
---

## Context
Answer-mode runs existed but gave no feedback.

## Description
Answer input in rehearsal.

## Acceptance Criteria
- [x] Submitting a typed answer asks for a comparison; the result shows the match, the verdict and the suggested rating
- [x] The user still rates by hand; a failed or late comparison never blocks rating
- [x] The service compares against a card (back) or a question (reference, accepted, rubric, min_score)

## Design Notes
`service::compare`, `Effect::ScoreAnswer`, `Msg::AnswerScored`. The score is not stored with the run event yet (that needs a schema change; see STATUS).

## Dependencies
TASK-0052.

## Open Questions
None.
