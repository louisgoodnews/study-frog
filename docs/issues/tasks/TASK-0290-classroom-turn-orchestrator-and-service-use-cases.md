---
id: TASK-0290
type: task
title: "Classroom turn orchestrator and service use cases in studyfrog-service"
status: done
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
- [x] `studyfrog-service` provides use cases to start a classroom session from vault stacks/notes/PDFs, advance a lecture segment, generate a simulated student question, and handle a user message
- [x] Every turn persists immediately to `studyfrog-store` and works with both local Ollama and remote providers via the existing `Provider` abstraction
- [x] Integration tests using a fake model verify multi-turn lectures, student Q&A, user interjections, and persistence

## Design Notes
- Respect existing remote-provider consent (`provider.allowed`) before sending vault text to a non-local endpoint.
- Build vs buy: the PDF source id is a 128-bit FNV-1a fingerprint of the vault path (about 15 lines, stable, not cryptographic); a hashing crate would add a dependency for that alone. Lecture chunking and the budgeter reuse existing code.
- Rolling summary: `ClassroomSession` has no summary field, so the orchestrator passes an empty summary and bounds the prompt with the recent-turn window. Persisting a summary needs a schema change, left for a follow-up.
- Persistence: each turn rewrites the session through `update_session` (one transaction). Fine at classroom sizes; measure before adding a row-level append.

## Dependencies
- `STORY-0209`, `TASK-0288`, `TASK-0289`

## Open Questions
None.
