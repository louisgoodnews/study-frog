---
id: STORY-0203
type: story
title: "Provisional budgets replaced by measured baselines on the owner's machine"
status: in_progress
epic: EPIC-067
parents: ['EPIC-067']
estimate: 2
labels: [m14, perf, docs]
acceptance_criteria:
  - 'Each provisional budget in BENCHMARKING.md is either supported by the owner''s baseline or changed in the document with the reason'
  - 'The baselines are recorded per machine, with the machine identity of each run'
  - 'The spread that sets the 5% floor of the regression rule is measured on the owner''s machine (10 back-to-back runs)'
---

## Context
The budgets in BENCHMARKING.md are marked provisional: they were derived from reasoning, not measured. EPIC-067
requires each one to be replaced by a measured baseline, or changed with the reason.

## Description
Once STORY-0201 has produced runs on the owner's machine, read the medians against each budget, update the
budget or the baseline table in BENCHMARKING.md, and record the machine identity beside each number.

## Acceptance Criteria
- [ ] Each provisional budget is either supported by the owner's baseline or changed in the document, with the reason
- [ ] The baselines are recorded per machine, with the machine identity of each run
- [ ] The spread that sets the 5% floor is measured on the owner's machine (EPIC-067 Open Questions)

## Progress (2026-11-24, reference machine per ADR 0014)
- The reference run (STORY-0201) is done. Each budget was read against its median in BENCHMARKING.md, section
  "Reference run on the 1.0.0 reference machine". Within budget: FSRS 100k, cold reindex, graph layout step (close to 8 ms),
  rehearsal per step, reports, analytics except CMRR, PDF, familiarity.
- Over budget here: analytics CMRR, 5,000 cards, 12 targets, 180 days: 84.8 ms against a budget of under 50 ms. No budget
  changed: this is the owner's decision (the number may be the machine or the code).
- Spread: the 5% floor is not met by 12 of 28 cases even after 10 runs. The idle-spread measurement (10 back-to-back runs,
  EPIC-067 Open Questions) is not done.
- Criteria 1 and 3 stay open. Criterion 2 (per-machine baseline with identity) holds for this machine only.

## Design Notes
- This story is left open on purpose. No budget number is changed without the owner's measurement, and this
  container is not a reference machine.
- Only the numbers the owner's runs support may change. The budget table keeps its existing rows and provenance column.

## Dependencies
Blocked on the owner's machine: the reference runs of STORY-0201 (TASK-0278 and TASK-0279), and the spread
measurement in EPIC-067 Open Questions. Unblocks when those runs exist.

## Open Questions
- Whether the owner's machine should be the only reference, or whether a second machine gets its own baseline table.
