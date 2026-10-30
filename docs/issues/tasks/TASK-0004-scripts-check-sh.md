---
id: TASK-0004
type: task
title: scripts/check.sh
status: done
epic: EPIC-001
parents: [STORY-0002]
estimate: 2
labels: [m0, tooling]
acceptance_criteria:
  - "Runs fmt --check, clippy -D warnings, test, linecount, check-deps, check-impls"
  - "Exits non-zero on first failure"
---

## Context
Part of STORY-0002.

## Description
scripts/check.sh.

## Acceptance Criteria
- [x] Runs fmt --check, clippy -D warnings, test, linecount, check-deps, check-impls
- [x] Exits non-zero on first failure

## Design Notes
Uses `--all-targets` for clippy so benches and tests are linted later.

## Dependencies
Parent story STORY-0002.

## Open Questions
None.
