---
id: TASK-0094
type: task
title: Global search
status: done
epic: EPIC-020
parents: [STORY-0036]
estimate: 3
labels: [m8, ui]
acceptance_criteria:
  - "Typing in the bar searches item titles across the vault (case-insensitive, prefix matches first, wildcards literal, at most 10)"
  - "Choosing a result, or pressing Enter for the first, opens it; Escape clears; stale answers are dropped"
---

## Context


## Description
Global search.

## Acceptance Criteria
- [x] Typing in the bar searches item titles across the vault (case-insensitive, prefix matches first, wildcards literal, at most 10)
- [x] Choosing a result, or pressing Enter for the first, opens it; Escape clears; stale answers are dropped

## Design Notes
`store::search_items`, `Effect::SearchItems`, `ChromeMsg::SearchResults`. Titles only: no body text or tags yet.

## Dependencies
TASK-0093.

## Open Questions
None.
