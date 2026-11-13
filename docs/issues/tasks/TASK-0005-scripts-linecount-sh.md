---
id: TASK-0005
type: task
title: "scripts/linecount.sh"
status: done
epic: EPIC-001
parents: [STORY-0002]
estimate: 1
labels: [m0, tooling]
acceptance_criteria:
  - "Lists `.rs` files over the limit (default 250), exit 1 if any"
  - "Locale-independent"
---

## Context
Part of STORY-0002.

## Description
scripts/linecount.sh.

## Acceptance Criteria
- [x] Lists `.rs` files over the limit (default 250), exit 1 if any
- [x] Locale-independent

## Design Notes
Counts per file rather than parsing `wc` totals (found a locale bug during demo).

## Dependencies
Parent story STORY-0002.

## Open Questions
None.
