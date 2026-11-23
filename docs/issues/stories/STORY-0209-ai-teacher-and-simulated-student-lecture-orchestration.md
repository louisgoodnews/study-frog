---
id: STORY-0209
type: story
title: "AI teacher and simulated student lecture orchestration (local and remote models)"
status: done
epic: EPIC-068
parents: ['EPIC-068']
estimate: 5
labels: [m14, agent, slm]
acceptance_criteria:
  - 'Lecture generation grounds teacher explanations and simulated student questions in selected vault notes, stacks, or lecture PDFs'
  - 'Orchestration works with both local models (e.g. Ollama) and configured remote providers through studyfrog-agent, respecting provider consent settings'
  - 'Context budgeting (rolling summary plus recent turns) keeps prompts bounded for small local models'
  - 'Users can ask questions or interject at any point and receive contextual teacher and peer responses'
---

## Context
A convincing classroom simulation requires coordinated turns: a teacher explaining material in digestible segments, simulated students asking realistic clarifying or probing questions, and the user jumping in at any moment—all working reliably on both small local models (like Ollama `qwen2.5:1.5b`) and larger remote models.

## Description
Implement the prompt builders and turn orchestrator across `studyfrog-agent` and `studyfrog-service`:
- Extract and chunk syllabus context from selected vault stacks, notes, or PDFs.
- Build role-specific system prompts for the teacher persona and each simulated student persona (e.g. curious beginner, detail-oriented skeptic, practical applier).
- Manage rolling lecture summaries and sliding recent-turn windows so token usage stays within local-model budgets.
- Orchestrate turn progression: next lecture segment, simulated student question, teacher answer, and user interjection.

## Acceptance Criteria
- [x] Lecture generation grounds teacher explanations and simulated student questions in selected vault notes, stacks, or lecture PDFs
- [x] Orchestration works with both local models (e.g. Ollama) and configured remote providers through `studyfrog-agent`, respecting provider consent settings
- [x] Context budgeting (rolling summary plus recent turns) keeps prompts bounded for small local models
- [x] Users can ask questions or interject at any point and receive contextual teacher and peer responses

## Design Notes
- Reuse `studyfrog-service::agent::ask` and existing `Provider` configuration so no new HTTP client or wire format is introduced.
- Test orchestration deterministically using fake/stub model closures in unit and integration tests.

## Dependencies
- `EPIC-068`, `STORY-0208`
- Child tasks: `TASK-0289`, `TASK-0290`

## Open Questions
- Whether simulated students should share the teacher's selected model by default with an optional per-role override later.

## Verification note

All four criteria are ticked on the strength of the tests in `studyfrog-service` (`classroom_start`, `classroom_lecture`, `classroom_provider`) and `studyfrog-agent` (`lecture_*`). The provider criterion is verified with a fake model and stub transport only; a run against a live Ollama instance or a configured remote provider has not been done.
