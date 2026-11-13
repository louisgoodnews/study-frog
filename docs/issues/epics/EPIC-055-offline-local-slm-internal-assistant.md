---
id: EPIC-055
type: epic
title: "Offline Local SLM Internal Assistant"
status: backlog
parents: []
labels: [m13, slm, agent]
acceptance_criteria:
  - 'Managed local background sidecar supervisor runs Qwen2.5-1.5B-Instruct on loopback port'
  - 'Automatic 5-minute idle shutdown ensures 100% RAM reclamation (0 MB idle footprint)'
  - 'Local on-device RAG queries SQLite vault embeddings for grounded answers'
  - 'Strict 1024-token prompt budgeting limits CPU prefill latency to under 3.5 seconds'
  - 'Streaming token generation delivers real-time assistant responses via SSE'
---

## Context
Students require intelligent tutoring, card extraction, and question answering without cloud subscriptions, paid API keys, or privacy leakage.

## Description
Deliver a permanent, fully offline internal AI assistant powered by a managed local background sidecar running Qwen2.5-1.5B with on-device RAG and strict CPU prompt budgeting.

## Acceptance Criteria
- [ ] Managed local background sidecar supervisor runs Qwen2.5-1.5B-Instruct on loopback port
- [ ] Automatic 5-minute idle shutdown ensures 100% RAM reclamation (0 MB idle footprint)
- [ ] Local on-device RAG queries SQLite vault embeddings for grounded answers
- [ ] Strict 1024-token prompt budgeting limits CPU prefill latency to under 3.5 seconds
- [ ] Streaming token generation delivers real-time assistant responses via SSE

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
None.

## Open Questions
None.
