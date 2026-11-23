---
id: TASK-0290
type: task
title: "Classroom turn orchestrator and service use cases in studyfrog-service"
status: backlog
epic: EPIC-068
parents: ['STORY-0209']
estimate: 2
labels: [m14, agent, slm]
acceptance_criteria:
  - 'studyfrog-service provides use cases to start a classroom session from vault stacks/notes/PDFs, advance a lecture segment, generate a simulated student question, and handle a user message'
  - 'Every turn persists immediately to studyfrog-store and works with both local Ollama and remote providers via the existing Provider abstraction'
  - 'Integration tests using a fake model verify multi-turn lectures, student Q&A, user interjections, and persistence'
---

## Context
Part of `STORY-0209`. Service-layer use cases compose vault content extraction (`notes_chunks` / `slide_chunks`), `studyfrog-agent` prompts, model execution, and `studyfrog-store` persistence.

## Description
Add `classroom` orchestration functions in `studyfrog-service` supporting:
- Session creation grounded in a stack, selected notes, or a lecture PDF.
- Advancing the teacher's next lecture segment.
- Generating a simulated student question and the teacher's reply.
- Processing a user question/comment and persisting each turn atomically.

## Acceptance Criteria
- [ ] `studyfrog-service` provides use cases to start a classroom session from vault stacks/notes/PDFs, advance a lecture segment, generate a simulated student question, and handle a user message
- [ ] Every turn persists immediately to `studyfrog-store` and works with both local Ollama and remote providers via the existing `Provider` abstraction
- [ ] Integration tests using a fake model verify multi-turn lectures, student Q&A, user interjections, and persistence

## Design Notes
- Respect existing remote-provider consent (`provider.allowed`) before sending vault text to a non-local endpoint.

## Dependencies
- `STORY-0209`, `TASK-0288`, `TASK-0289`

## Open Questions
None.
