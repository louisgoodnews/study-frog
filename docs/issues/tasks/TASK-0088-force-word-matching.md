---
id: TASK-0088
type: task
title: "Setting to compare answers by word matching only"
status: done
epic: EPIC-019
parents: [STORY-0035]
estimate: 3
labels: [m6, followup]
acceptance_criteria:
  - "answer_engine = auto or words in app.toml, and a choice on the Settings screen"
  - "With words the language model is never loaded"
---

## Context
An open flag from STATUS.md.

## Description
Setting to compare answers by word matching only.

## Acceptance Criteria
- [x] answer_engine = auto or words in app.toml, and a choice on the Settings screen
- [x] With words the language model is never loaded

## Design Notes
`AnswerEngine` in core, parsed by config, honoured per comparison by the executor.

## Dependencies
None.

## Open Questions
None.
