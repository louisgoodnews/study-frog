---
id: STORY-0221
type: story
title: "Choice grading in rehearsal: reuse the mock exam rules and record the result like a typed answer"
status: review
epic: EPIC-072
parents: ['EPIC-072']
estimate: 5
labels: [m14, rehearsal]
acceptance_criteria:
  - "The rehearsal engine grades a choice answer with the mock exam's SingleSelect and MultipleSelect rules from studyfrog-rehearsal exam_answer.rs (no second implementation)"
  - "A correct choice suggests a good rating and a wrong one a again rating, as a typed answer does; the result is recorded in the same run record"
  - "Unit tests cover single correct, single wrong, multiple all correct, multiple with one missing, multiple with one wrong"
---

## Context
Grading exists for the mock exam; rehearsal needs the same rules for ordinary items.

## Description
Connect a rehearsal choice item to the existing objective grading, and record the outcome in the run like a typed answer.

## Acceptance Criteria
- [ ] Reuses exam_answer grading
- [ ] Suggested rating and recording follow typed answers
- [ ] Tests for the five cases above

## Design Notes
Keep the rehearsal engine's action names unchanged; add the choice variant to the existing answer path.

## Dependencies
- STORY-0220.

## Open Questions
None.
