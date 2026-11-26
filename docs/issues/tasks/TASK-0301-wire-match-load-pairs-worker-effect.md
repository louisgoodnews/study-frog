---
id: TASK-0301
type: task
title: "Wire MatchEffect::LoadPairs worker job and remove biology demo fallback"
status: done
epic: EPIC-075
parents: ['STORY-0244']
estimate: 2
labels: [m14, ui]
acceptance_criteria:
  - 'Navigating to Screen::Match or changing scope dispatches MatchEffect::LoadPairs on the worker and populates MatchUiState with real vault pairs'
  - 'view/match_grid.rs no longer dispatches default_match_pairs() on live vaults; default_match_pairs() is reserved for tests only'
  - 'MatchMsg::NewRound samples a fresh randomized subset from the loaded vault pool'
---

## Context
Part of `STORY-0244`. Connects `studyfrog-service`'s pair loader (`TASK-0300`) to `studyfrog-ui`'s Elm-style update loop and worker.

## Description
Add `MatchEffect::LoadPairs` and `MatchMsg::PairsLoaded(Result<Vec<(String, String)>, String>)` in `studyfrog-ui`, execute it on the worker, and trigger it automatically on navigating to `Screen::Match` or updating scope/filter.

## Acceptance Criteria
- [x] Navigating to `Screen::Match` or changing scope dispatches `MatchEffect::LoadPairs` on the worker and populates `MatchUiState` with real vault pairs
- [x] `view/match_grid.rs` no longer dispatches `default_match_pairs()` on live vaults; `default_match_pairs()` is reserved for tests only
- [x] `MatchMsg::NewRound` samples a fresh randomized subset from the loaded vault pool

## Design Notes
- Preserve clean `Effect` separation (no blocking I/O on the UI thread).

## Dependencies
- `STORY-0244`, `TASK-0300`

## Open Questions
None.
