---
id: TASK-0003
type: task
title: "Dependency-direction check script"
status: done
epic: EPIC-001
parents: [STORY-0001]
estimate: 2
labels: [m0, tooling]
acceptance_criteria:
  - "`scripts/check-deps.sh` fails on a forbidden edge (demonstrated core->ui)"
  - "Allowed edges match ADR 0001"
---

## Context
Part of STORY-0001.

## Description
Dependency-direction check script.

## Acceptance Criteria
- [x] `scripts/check-deps.sh` fails on a forbidden edge (demonstrated core->ui)
- [x] Allowed edges match ADR 0001

## Design Notes
Allowed-edge table is in the script; update it together with ADR 0001.

## Dependencies
Parent story STORY-0001.

## Open Questions
Using `cargo metadata` instead of manifest parsing would also catch renamed deps.
