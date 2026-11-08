---
id: STORY-0129
type: story
title: Exam mode help and control protocol
status: done
epic: EPIC-033
parents: [EPIC-033]
estimate: 3
labels: [m12, docs, ui, app]
acceptance_criteria:
  - "The in-app help has an Exam mode page (rules, scoring, grade table, result) linked from the Rehearsing page, the keyboard page and the FAQ"
  - "studyfrog --control-stdio and studyfrog ctl can set up an exam (rehearse setup --mode exam), start it, type and submit answers (rehearse type, rehearse submit, rehearse answer), read the locked answers and the result in state, and finish (rehearse finish); refused actions answer with an error; tested in a headless session against a real vault"
---
## Context
An agent or a learner needs to know the rules; agents need to drive an exam like the window does.

## Description
Help page (TASK-0241); control commands (TASK-0242).

## Acceptance Criteria
- [x] The in-app help has an Exam mode page (rules, scoring, grade table, result) linked from the Rehearsing page, the keyboard page and the FAQ
- [x] studyfrog --control-stdio and studyfrog ctl can set up an exam (rehearse setup --mode exam), start it, type and submit answers (rehearse type, rehearse submit, rehearse answer), read the locked answers and the result in state, and finish (rehearse finish); refused actions answer with an error; tested in a headless session against a real vault

## Design Notes
`docs/help/exam-mode.md`; `control_words.rs`, `control_plan.rs`, `control_parts.rs`, `control_help.rs`.

## Dependencies
STORY-0128, STORY-0124.

## Open Questions
None.
