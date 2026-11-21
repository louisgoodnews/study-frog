---
id: STORY-0177
type: story
title: "Adaptive Cognitive Difficulty and Learning Velocity Estimation Engine"
status: review
epic: EPIC-060
parents: ['EPIC-060']
estimate: 5
labels: [m14, srs, report]
acceptance_criteria:
  - 'Pure mathematical engine computing empirical item difficulty from latency, dwell time, and semantic accuracy'
  - 'User-level learning velocity estimator quantifying personal memory consolidation rate over time'
  - 'Automated calibration of initial item difficulty priors (D0) based on cognitive telemetry'
---

## Context
Fixed difficulty baselines fail to reflect that different learners process conceptual density, technical terminology, and formulaic tasks at vastly different speeds. Empirical telemetry allows computing personalized difficulty and learning velocity.

## Description
Develop a mathematical engine in `studyfrog-report` (or `studyfrog-srs`):
- Compute an empirical difficulty index $D_{\text{emp}} \in [1, 10]$ derived from normalized response latency, relative dwell time compared to text length, and SLM semantic correctness.
- Formulate a learning velocity metric: the rate at which stability $S$ grows per unit of study time invested.
- Feed calibrated item difficulty into initial memory state estimation (`initial_state`), preventing over-scheduling of deceptively long or under-recalled cards.

## Acceptance Criteria
- [x] Pure mathematical engine computing empirical item difficulty from latency, dwell time, and semantic accuracy
- [x] User-level learning velocity estimator quantifying personal memory consolidation rate over time
- [ ] Automated calibration of initial item difficulty priors (D0) based on cognitive telemetry

## Design Notes
Implement strictly as free functions adhering to Invariant P1 (zero inherent `impl` blocks). Maintain deterministic numerical stability with thorough unit and property tests.

## Dependencies
EPIC-060, STORY-0176

## Implementation (2026-11-21)
`studyfrog-report::difficulty`: D_emp on [1, 10], the velocity slope, and `calibrated_prior` (the prior counts as five observations). Tests: `tests/difficulty.rs`. Open: feeding the calibrated D0 into `initial_state` in `studyfrog-srs`, and the velocity chart in the report.

## Open Questions
None.
