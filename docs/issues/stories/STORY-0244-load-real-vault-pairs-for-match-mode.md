---
id: STORY-0244
type: story
title: "Load real front/back pairs from vault flashcards and questions for Match mode"
status: done
epic: EPIC-075
parents: ['EPIC-075']
estimate: 5
labels: [m14, rehearse, service]
acceptance_criteria:
  - 'studyfrog-service extracts real prompt/answer pairs from vault Cards (front/back) and Questions (prompt/answer) filtered by scope (entire vault, selected stacks, or due/weak items)'
  - 'Markdown formatting is stripped cleanly into concise tile text and empty or duplicate pairs are skipped'
  - 'Opening Match mode or changing scope dispatches an async worker effect that loads vault pairs instead of hardcoded biology demo cards'
  - 'Dealing a new round samples a fresh randomized subset of pairs from the loaded deck'
---

## Context
Match mode (`EPIC-075`) currently dispatches `default_match_pairs()` (hardcoded biology cards) on screen entry and its unused `StartFromVault` handler only fabricates `"Answer for <title>"` strings from item titles.

## Description
Build real vault pair loading across `studyfrog-service` and `studyfrog-ui`:
- Service-layer pure extraction of `Card` front/back and `Question` prompt/answer pairs with scope filtering and Markdown cleanup.
- Async worker effect (`MatchEffect::LoadPairs`) wired into `studyfrog-ui` so Match mode loads live vault pairs automatically on navigation and scope changes.

## Acceptance Criteria
- [x] `studyfrog-service` extracts real prompt/answer pairs from vault Cards (front/back) and Questions (prompt/answer) filtered by scope (entire vault, selected stacks, or due/weak items)
- [x] Markdown formatting is stripped cleanly into concise tile text and empty or duplicate pairs are skipped
- [x] Opening Match mode or changing scope dispatches an async worker effect that loads vault pairs instead of hardcoded biology demo cards
- [x] Dealing a new round samples a fresh randomized subset of pairs from the loaded deck

## Design Notes
- Keep `default_match_pairs()` strictly for tests/fixtures, never as a silent runtime fallback when a vault is open.
- Follow Invariant P1 (pure free functions) and P6 ($\le 250$ LOC per file).

## Dependencies
- `EPIC-075`
- Child tasks: `TASK-0300`, `TASK-0301`

## Open Questions
None.
