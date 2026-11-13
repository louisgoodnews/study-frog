---
id: STORY-0164
type: story
title: "On-Device Vault RAG Pipeline and CPU Prompt Budgeting"
status: backlog
epic: EPIC-055
parents: ['EPIC-055']
estimate: 5
labels: [m13, service]
acceptance_criteria:
  - 'SQLite table vault_embeddings stores 384-dimensional MiniLM note embeddings'
  - 'Cosine vector similarity ranking retrieves top relevant vault snippets in sub-15ms'
  - 'Strict 1024-token prompt budgeting limits CPU prefill latency to under 3.5 seconds'
  - "UI displays distinct 'Thinking / Analyzing vault...' feedback during prefill phase"
---

## Context
Grounding assistant answers in user notes without exceeding CPU compute limits.

## Description
Implement local on-device RAG: embed notes with MiniLM, store vectors in SQLite, assemble top-3 context snippets, and enforce strict 1024-token prompt budgeting.

## Acceptance Criteria
- [ ] SQLite table vault_embeddings stores 384-dimensional MiniLM note embeddings
- [ ] Cosine vector similarity ranking retrieves top relevant vault snippets in sub-15ms
- [ ] Strict 1024-token prompt budgeting limits CPU prefill latency to under 3.5 seconds
- [ ] UI displays distinct 'Thinking / Analyzing vault...' feedback during prefill phase

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-055

## Open Questions
None.
