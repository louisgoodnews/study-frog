---
id: TASK-0307
type: task
title: "Stats-driven preparation-window estimator (propose_prep_weeks)"
status: done
epic: EPIC-076
parents: ['STORY-0247']
estimate: 3
labels: [m14, srs, report]
acceptance_criteria:
  - 'A pure free function propose_prep_weeks computes a recommended week interval from item count, unstudied share, average familiarity, FSRS stability/lapse stats, and daily study capacity'
  - 'Returns both the proposed week count (clamped to available time before exam_date) and human-readable factor explanations'
  - 'Unit and property tests verify monotonic behavior (more items or lower familiarity never proposes fewer weeks) and edge cases (0 items, exam tomorrow)'
---

## Context
Part of `STORY-0247`. Enables StudyFrog to propose an evidence-based preparation window in weeks rather than forcing the student to guess.

## Description
Implement `propose_prep_weeks` as a pure function combining item count, familiarity, FSRS stability/difficulty, recent accuracy, and daily budget into a recommended week count plus a concise list of human-readable reasons.

## Acceptance Criteria
- [x] A pure free function `propose_prep_weeks` computes a recommended week interval from item count, unstudied share, average familiarity, FSRS stability/lapse stats, and daily study capacity
- [x] Returns both the proposed week count (clamped to available time before `exam_date`) and human-readable factor explanations
- [x] Unit and property tests verify monotonic behavior (more items or lower familiarity never proposes fewer weeks) and edge cases (0 items, exam tomorrow)

## Design Notes
- Deterministic with injected `today` date (P2); zero randomness.

## Dependencies
- `STORY-0247`, `TASK-0305`

## Open Questions
None.
