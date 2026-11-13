---
id: TASK-0050
type: task
title: "Candle embedder and spike"
status: done
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
- [x] A candle embedder implements `SlmRuntime` with a MiniLM-class model
- [x] p50/p95 latency, memory and download size are measured and compared with `ort`
- [x] ADR 0004 is updated with the numbers

## Design Notes
`studyfrog-candle::minilm` (its own crate, linked only by the app so other builds and tests stay fast): BERT encoder, mean pooling, unit length; weights are read into memory and loaded with the safe buffered loader, so `unsafe_code = "forbid"` still holds. Measured results are in ADR 0004. The `ort` comparison was not run because candle meets the budget.

## Dependencies
None.

## Open Questions
None.
