---
id: STORY-0199
type: story
title: "Case studies in the rehearsal run queue"
status: backlog
epic: EPIC-061
parents: ['EPIC-061']
estimate: 5
labels: [m14, rehearse, ui]
acceptance_criteria:
  - 'A rehearsal run can include case studies of the chosen stacks, interleaved with cards and questions by the scheduler'
  - 'A case is answered in the run: choice, justification, and the rubric evaluation shown before the next item'
  - 'The run records each case answer like a question answer, and the case is scheduled from its rating'
---

## Context
Case studies are practised in their own view, reached from the rehearse setup. A learner who is rehearsing a stack should meet its cases in the same run as its other items, so the schedule covers them too.

## Description
- Include the case studies of the selected stacks in the run queue (`studyfrog-service` queue building, `studyfrog-rehearsal` run states).
- Show the case inside the run's item view: vignette, decision and options, justification, and the evaluation (`view::case_view` is the starting point).
- Map the evaluation to a rating (`suggested_rating` in `studyfrog-slm` takes a verdict) and record it as the run's answer.

## Acceptance Criteria
- [ ] A rehearsal run can include case studies of the chosen stacks, interleaved with cards and questions by the scheduler
- [ ] A case is answered in the run: choice, justification, and the rubric evaluation shown before the next item
- [ ] The run records each case answer like a question answer, and the case is scheduled from its rating

## Design Notes
The case view currently works outside a run (`setup::case`). The run item view needs the same widgets; share them rather than copy. Keep files under 250 lines.

## Dependencies
EPIC-061, STORY-0179, STORY-0181, STORY-0200

## Open Questions
- Should a case count as one item or as its own card-like unit in the run's daily limits?
- Which rating maps from a rubric score (the thresholds in `suggested_rating` were written for other verdicts)?
