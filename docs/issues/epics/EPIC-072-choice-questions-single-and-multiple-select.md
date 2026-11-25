---
id: EPIC-072
type: epic
title: "Choice questions in rehearsal: single choice and multiple select"
status: backlog
parents: []
labels: [m14, rehearsal, md, ui]
acceptance_criteria:
  - "A card or question can list options, marked as correct or not, in plain Markdown; one correct option makes a single-choice item, more than one a multiple-select item"
  - "Rehearsal shows the options, grades the choice (single: exactly the correct option; multiple: all correct options and no wrong ones) and records the result like a typed answer"
  - "The choice is answered by mouse, by the number keys and through the control protocol; the answer and the correct options are shown afterwards"
  - "The same grading as the mock exam (SingleSelect and MultipleSelect in studyfrog-rehearsal exam_answer.rs) is reused, not reimplemented"
---

## Context
The mock exam (Altklausur) already generates and grades single-select and multiple-select questions. Ordinary rehearsal
items (cards, questions) cannot be choice questions yet. The owner asked for this as a new issue.

## Description
Make a choice item an ordinary rehearsal item: a Markdown list of options with `[x]` marking the correct ones, parsed by
studyfrog-md, graded by the rehearsal engine with the mock exam's rules, and shown by the rehearsal card.

## Acceptance Criteria
- [ ] Options in Markdown; one `[x]` gives single choice, more gives multiple select
- [ ] Rehearsal shows, grades and records the choice; the correct options are shown afterwards
- [ ] Mouse, number keys and the control protocol all answer the choice
- [ ] The mock exam's grading is reused

## Design Notes
- Syntax (proposal, decided in STORY-0220): a list whose items start with `- [ ]` (wrong) or `- [x]` (correct), placed
  in the body of a card or question, after the prompt. Example:
  ```
  Which organelle makes ATP?
  - [ ] Ribosome
  - [x] Mitochondrion
  - [ ] Golgi apparatus
  ```
- Multiple select: grade all-or-nothing by default (every correct option chosen, no wrong option); a partial score can be
  added later.

## Dependencies
- studyfrog-md (parser), studyfrog-rehearsal (grading, exam_answer.rs), studyfrog-ui (rehearsal card), control protocol.

## Open Questions
- Partial credit for multiple select: all-or-nothing for now (see Design Notes).
