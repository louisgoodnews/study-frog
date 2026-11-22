---
id: TASK-0281
type: task
title: "bench-compare command, wrapper script, and the repo-lint tests in the gate"
status: done
epic: EPIC-067
parents: ['STORY-0202']
estimate: 1
labels: [m14, perf, tooling]
acceptance_criteria:
  - 'repo-lint bench-compare <base.ns> <new.ns> prints one verdict line and exits 1 only for a regression, 2 for bad input'
  - 'scripts/bench-compare.sh wraps it, and scripts/check.sh runs the repo-lint tests'
---

## Context
Part of STORY-0202.

## Description
A subcommand of repo-lint, which already holds the repository's lints and their tests. The gate now runs its tests
(`cargo test` on tools/repo-lint), so the rule's tests cannot silently stop running.

## Acceptance Criteria
- [x] The subcommand prints one line and exits 1 only for a regression, 2 for bad input
- [x] The wrapper script exists, and check.sh runs the repo-lint tests (verified in the gate run)

## Design Notes
- Checked here: two halves of one run gave no change (ratio 0.988); the same run 25% slower gave no change too,
  because its 9.8% spread sets the floor at 29%. That is the rule working, not a fault. Bad usage exited 2.

## Dependencies
STORY-0202, TASK-0280.

## Open Questions
None.
