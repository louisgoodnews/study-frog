---
id: EPIC-075
type: epic
title: "Make Match mode usable: source real front/back pairs from vault flashcards and questions with stack selection and playable tile layout"
status: backlog
parents: []
labels: [m14, rehearse, service, ui]
acceptance_criteria:
  - 'Opening Match mode loads real prompt/answer pairs (card front/back and question prompt/answer) from the active vault via studyfrog-service instead of hardcoded biology demo cards'
  - 'Users can choose the source scope before or between rounds (entire vault, one or more selected stacks, or due/weak items) and configure pair count / grid size (e.g. 4, 6, 8 pairs)'
  - 'Starting a new round or clicking "Next round" draws a fresh randomized set of pairs from the chosen scope rather than repeating only the same 6 cards, while "Restart" reshuffles the current round'
  - 'Match tiles handle real-world vault card text cleanly (dynamic sizing or readable wrapping/truncation with full-text hover preview, clear prompt vs answer distinction, and responsive grid filling the screen width)'
  - 'When the chosen scope has too few cards/questions to start a match, the screen shows a clear empty state explaining what is needed instead of silently substituting unrelated demo biology cards'
---

## Context
Currently, opening **Match** from the left sidebar always shows six hardcoded biology demo pairs (*Mitochondria*, *Ribosome*, *Golgi Apparatus*, *Lysosome*, *Chloroplast*, *Endoplasmic Reticulum*) even when a real vault with dozens of flashcards is open:
- `crates/studyfrog-ui/src/view/match_grid.rs:175` unconditionally dispatches `MatchMsg::Start(default_match_pairs())` whenever `state.match_state` is `None`.
- Even the unused `MatchMsg::StartFromVault` branch in `crates/studyfrog-ui/src/match_ui.rs:139-153` only inspects `state.browser.items` (which has no card back/answer text) and fabricates dummy answers (`format!("Answer for {}", item.title)`), falling back to `default_match_pairs()` whenever fewer than 6 items are present.
- Tiles are fixed at `max 240x80` in a left-aligned 3x4 grid with no clipping/overflow protection for longer flashcard fronts or answers, no stack selector, and no way to draw a fresh set of pairs from a larger deck.

## Description
Turn Match mode into a genuinely useful active-recall drill backed by the user's real vault:
1. **Real Vault Pair Loading (`studyfrog-service`, `studyfrog-ui`)**:
   - Add a service use case that reads actual `Card` (front/back) and `Question` (prompt/answer) content from the vault (filtered by whole vault, selected stack(s), or due/all items), stripping Markdown syntax cleanly into readable tile text and skipping empty/malformed pairs.
   - Load pairs asynchronously on the worker via a `MatchEffect::LoadPairs { ... }` when entering Match mode or changing scope.
2. **Scope Picker & Round Controls (`studyfrog-ui`)**:
   - Provide a compact setup/controls bar (or setup step) allowing the user to pick:
     - **Scope:** Entire vault or specific stack(s) (with item counts).
     - **Filter:** All cards/questions, Due/overdue first, or Least familiar.
     - **Pairs per round:** e.g. 4, 6, or 8 pairs.
   - Distinguish **Restart** (reshuffle current round's pairs) from **New round / Deal next** (sample a fresh subset of pairs from the deck) and fix the `Exit` button tooltip (currently says *"Quits the app."*).
3. **Readable, Responsive Tile Grid (`studyfrog-ui`)**:
   - Center and scale the grid cleanly across the available viewport width with taller/adaptive tiles, clean wrapping/ellipsizing inside tile bounds (so long prompts/answers never spill outside the tile border), and full-text hover tooltips when truncated.
   - Visually distinguish or subtly badge prompt vs answer tiles if helpful, and keep mismatch/selected/matched feedback crisp and accessible.
4. **Honest Empty State & Session Recording**:
   - If the selected scope has fewer than 2 valid card/question pairs, show a clear empty state with a button to switch scope or create cards—never silently load biology demo cards.
   - Persist personal best times/scores per scope (or session) and optionally record match completions in activity/reports.

## Acceptance Criteria
- [ ] Opening Match mode loads real prompt/answer pairs (card front/back and question prompt/answer) from the active vault via `studyfrog-service` instead of hardcoded biology demo cards
- [ ] Users can choose the source scope before or between rounds (entire vault, one or more selected stacks, or due/weak items) and configure pair count / grid size (e.g. 4, 6, 8 pairs)
- [ ] Starting a new round or clicking "Next round" draws a fresh randomized set of pairs from the chosen scope rather than repeating only the same 6 cards, while "Restart" reshuffles the current round
- [ ] Match tiles handle real-world vault card text cleanly (dynamic sizing or readable wrapping/truncation with full-text hover preview, clear prompt vs answer distinction, and responsive grid filling the screen width)
- [ ] When the chosen scope has too few cards/questions to start a match, the screen shows a clear empty state explaining what is needed instead of silently substituting unrelated demo biology cards

## Design Notes
- Existing files:
  - `crates/studyfrog-rehearsal/src/match_mode.rs` (pure session state machine)
  - `crates/studyfrog-ui/src/match_ui.rs` (`MatchUiState`, `MatchMsg`, `update`)
  - `crates/studyfrog-ui/src/view/match_grid.rs` & `match_victory.rs`
- Keep all domain/sampling logic in pure free functions (Invariant P1), files $\le 250$ LOC (Invariant P6), theme tokens via `palette_of(ui)` (no raw RGB literals), and accessible widget names for headless UI tests and `studyfrog ctl`.
- Retain `default_match_pairs()` only for unit tests or explicit demo/screenshot fixtures, never as the live vault screen fallback.

## Dependencies
- Existing Match mode engine (`EPIC-059` / `EPIC-053`, done)

## Open Questions
- Should Match mode completions update FSRS/SRS scheduling or remain practice-only (like Memory mode)?
- Should Match mode also support matching cloze gaps (`prompt with [...]` ↔ `missing answer`) in addition to standard cards and questions?
