---
id: STORY-0220
type: story
title: "Choice item syntax: options in Markdown, parsed into single-choice or multiple-select items"
status: in_progress
epic: EPIC-072
parents: ['EPIC-072']
estimate: 3
labels: [m14, md, rehearsal]
acceptance_criteria:
  - "studyfrog-md parses a list of `- [ ]` and `- [x]` options in a card or question body into a choice block, keeping the prompt text before it"
  - "One correct option gives a single-choice item; two or more give a multiple-select item; no correct option is a parse problem that is reported, not guessed"
  - "The parsed options keep their order and their text, and are exposed as plain data (no inherent impl blocks)"
  - "Unit tests cover single, multiple, no correct option, and a list that is not a choice block"
---

## Context
EPIC-072 needs items to say which options exist and which are correct, in a form the vault can keep as plain text.

## Description
Add a pure parser in studyfrog-md for choice blocks, and the data type that carries them. Keep it independent of the UI.

## Acceptance Criteria
- [ ] Parses options and the prompt
- [ ] Single versus multiple decided by the number of correct options
- [ ] Reports a missing correct option
- [ ] Tests for each case

## Design Notes
Syntax as in EPIC-072. Use the existing studyfrog-md list parsing where possible.

## Dependencies
- EPIC-072.

## Open Questions
None.
