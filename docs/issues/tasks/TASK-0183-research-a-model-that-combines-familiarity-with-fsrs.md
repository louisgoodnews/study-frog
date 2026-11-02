---
id: TASK-0183
type: task
title: Research a model that combines familiarity with FSRS
status: done
epic: EPIC-023
parents: [STORY-0059]
estimate: 5
labels: [m10, srs, docs]
acceptance_criteria:
  - "docs/research/familiarity-and-fsrs.md proposes a model, compares it with alternatives and states its limits"
  - "A reproducible simulation (docs/research/familiarity-fsrs-sim.py) compares the candidates with plain FSRS at equal workload, and the port of FSRS-6 in it matches the Rust implementation"
  - "The document gives a plan for deciding on real data and the decisions the owner has to make"
---

## Context
The owner asked on 2026-11-02 that familiarity influence FSRS so that well-known cards are scheduled more generously (a card rated Easy at familiarity 0.5 should come back after about 2.5 months, not 5), and tasked the research and proposal of a model that combines the two. The 30 days mentioned for familiarity 1.0 were an example, never a cap.

## Description
Research and propose how familiarity and FSRS can be combined.

## Acceptance Criteria
- [x] docs/research/familiarity-and-fsrs.md proposes a model, compares it with alternatives and states its limits
- [x] A reproducible simulation (docs/research/familiarity-fsrs-sim.py) compares the candidates with plain FSRS at equal workload, and the port of FSRS-6 in it matches the Rust implementation
- [x] The document gives a plan for deciding on real data and the decisions the owner has to make

## Design Notes
Result: a relative multiplier on the FSRS interval, `m = exp(beta * (F - F_expected(S)))`, with the stored FSRS state untouched. The simulation supports the structure (relative beats absolute, and changing the stored state gains almost nothing) but does not show that ratings or view time add information beyond a time-based brake on interval growth; the model is therefore gated by a replay on the user's own history (TASK-0184). The Python port of FSRS-6 reproduces the Rust stability, difficulty and interval values for three rating sequences.

## Dependencies
None.

## Open Questions
The owner decided on 2026-11-02: relative reading, defaults, and on from the start (section 10 of the document).
