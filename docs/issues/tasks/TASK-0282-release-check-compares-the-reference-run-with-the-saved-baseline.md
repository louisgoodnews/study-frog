---
id: TASK-0282
type: task
title: "Release check: compare the reference run with the saved baseline before tagging"
status: backlog
epic: EPIC-067
parents: ['STORY-0202']
estimate: 1
labels: [m14, perf, docs]
acceptance_criteria:
  - 'The release section of BENCHMARKING.md lists the steps: reference run on the owner''s machine, bench-compare against the saved baseline for each case, and the verdict lines in the release notes'
  - 'A regression verdict blocks the release until it is either fixed or accepted in writing by the owner'
---

## Context
Part of STORY-0202: "a change that slows a hot path past its budget or its baseline is caught before release."

## Description
Document the step and where its output goes. The step is run by the owner, on the owner's machine, before tagging.

## Acceptance Criteria
- [ ] The release steps are in BENCHMARKING.md
- [ ] A regression verdict blocks the release unless the owner accepts it in writing

## Design Notes
- BENCHMARKING.md's budget numbers are not changed here (STORY-0203).

## Dependencies
TASK-0278 and TASK-0279 (runs on the owner's machine), STORY-0203 (baselines).

## Open Questions
- Whether the release checklist lives in BENCHMARKING.md or in the release process document.
