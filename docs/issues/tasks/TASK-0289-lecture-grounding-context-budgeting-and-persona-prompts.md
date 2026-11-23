---
id: TASK-0289
type: task
title: "Lecture grounding, context budgeting, and persona prompt builders in studyfrog-agent"
status: done
epic: EPIC-068
parents: ['STORY-0209']
estimate: 3
labels: [m14, agent, slm]
acceptance_criteria:
  - 'Pure prompt builders in studyfrog-agent construct system and user messages for the teacher persona, configurable simulated student personas, and rolling lecture summarization'
  - 'A token/character context budgeter packs syllabus excerpts, rolling summary, and recent turns without overflowing small local-model limits'
  - 'Unit tests verify prompt structure, role isolation, and budget truncation behavior'
---

## Context
Part of `STORY-0209`. Small local models (e.g. 1.5B–3B parameters) degrade quickly when chat history grows unbounded, so classroom prompts need strict context budgeting and clear persona framing.

## Description
Implement pure prompt-construction and budgeting functions in `studyfrog-agent`:
- Teacher prompt builder (progressive lecture explanation, answering user/student questions, checking comprehension).
- Simulated student prompt builder (distinct personas asking grounded, non-repetitive questions).
- Context budgeter combining syllabus chunks, rolling summary, and the most recent turns within a configurable budget.

## Acceptance Criteria
- [x] Pure prompt builders in `studyfrog-agent` construct system and user messages for the teacher persona, configurable simulated student personas, and rolling lecture summarization
- [x] A token/character context budgeter packs syllabus excerpts, rolling summary, and recent turns without overflowing small local-model limits
- [x] Unit tests verify prompt structure, role isolation, and budget truncation behavior

## Design Notes
- Keep `studyfrog-agent` depending only on `studyfrog-core` per ADR 0001 layering.

## Dependencies
- `STORY-0209`, `TASK-0287`

## Open Questions
None.
