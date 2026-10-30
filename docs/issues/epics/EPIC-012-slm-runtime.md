---
id: EPIC-012
type: epic
title: SLM runtime
status: in_progress
parents: []
labels: [m6, slm]
acceptance_criteria:
  - "Runtime swappable behind `SlmRuntime`"
  - "A semantic model runs offline within the latency budget"
---

## Context
Compare typed answers with the reference, locally.

## Description
SLM runtime.

## Acceptance Criteria
- [ ] Runtime swappable behind `SlmRuntime`
- [ ] A semantic model runs offline within the latency budget

## Design Notes
Children: STORY-0023. The lexical runtime ships first; the semantic runtime waits for the spike (ADR 0004).

## Dependencies
M4 exit.

## Open Questions
None.
