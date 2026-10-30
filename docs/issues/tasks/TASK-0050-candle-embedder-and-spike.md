---
id: TASK-0050
type: task
title: Candle embedder and spike
status: blocked
epic: EPIC-012
parents: [STORY-0023]
estimate: 8
labels: [m6, slm, spike]
acceptance_criteria:
  - "A candle embedder implements `SlmRuntime` with a MiniLM-class model"
  - "p50/p95 latency, memory and download size are measured and compared with `ort`"
  - "ADR 0004 is updated with the numbers"
---

## Context
The accepted ADR still needs its numbers.

## Description
Candle embedder and spike.

## Acceptance Criteria
- [ ] A candle embedder implements `SlmRuntime` with a MiniLM-class model
- [ ] p50/p95 latency, memory and download size are measured and compared with `ort`
- [ ] ADR 0004 is updated with the numbers

## Design Notes
Blocked: needs a model download (about 90 MB from Hugging Face) and the heavy candle dependency; waiting for the user to approve.

## Dependencies
None.

## Open Questions
None.
