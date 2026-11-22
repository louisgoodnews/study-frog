---
id: STORY-0199
type: story
title: "Case studies in the rehearsal run queue"
status: review
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
- [x] A rehearsal run can include case studies of the chosen stacks, interleaved with cards and questions by the scheduler
- [x] A case is answered in the run: choice, justification, and the rubric evaluation shown before the next item
- [x] The run records each case answer like a question answer, and the case is scheduled from its rating

## Implementation (2026-11-22)
A case study is a question item, so the run queue already includes it with the other items of the stack. The item content now carries the case (`studyfrog-service::content_case`), and the run view shows its options and the justification box in place of the typed-answer box. "Check my answer" checks the ticked option and the justification with the rubric check (`setup::case::run`), then reveals the answer with the usual flip. The check's result (choice pill and a met or missing pill per rubric point) shows under the answer, and its suggested rating is filled in on the rating buttons; the rating itself stays the student's.

The suggestion: by the overall score (Easy from 90 %, Good from 70 %, Hard from 40 %, otherwise Again); at most Hard when the choice is wrong; Again when no rubric point is made.

The check's score (value, verdict, engine) is stored for the item and run through `record_answer_score`, as a typed answer's score is (`AnswerEffect::Record`); the rating is recorded and schedules the item. The typed justification text is not stored, as for typed answers. The engine accepts typed answers only in answer modes, so the check does not go through the typed-answer path; the local model is not used inside a run (it is in the case view, STORY-0200).

Tests: `studyfrog-ui/tests/case_run.rs` (suggested rating), `studyfrog-service/tests/case_items.rs` (a run loads the case). A screenshot of the run (`--screen run` on a one-case vault) shows the case controls.

## Design Notes
The case view currently works outside a run (`setup::case`). The run item view needs the same widgets; share them rather than copy. Keep files under 250 lines.

## Dependencies
EPIC-061, STORY-0179, STORY-0181, STORY-0200

## Open Questions
- Should a case count as one item or as its own card-like unit in the run's daily limits?
- Which rating maps from a rubric score (the thresholds in `suggested_rating` were written for other verdicts)?
