---
id: STORY-0203
type: story
title: "Provisional budgets replaced by measured baselines on the owner's machine"
status: blocked
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

## Design Notes
- This story is left open on purpose. No budget number is changed without the owner's measurement, and this
  container is not a reference machine.
- Only the numbers the owner's runs support may change. The budget table keeps its existing rows and provenance column.

## Dependencies
Blocked on the owner's machine: the reference runs of STORY-0201 (TASK-0278 and TASK-0279), and the spread
measurement in EPIC-067 Open Questions. Unblocks when those runs exist.

## Open Questions
- Whether the owner's machine should be the only reference, or whether a second machine gets its own baseline table.
