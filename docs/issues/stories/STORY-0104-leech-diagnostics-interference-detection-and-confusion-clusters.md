---
id: STORY-0104
type: story
title: Leech diagnostics, interference detection, and confusion clusters
status: done
epic: EPIC-034
parents: [EPIC-034]
estimate: 5
labels: [m12, report, srs, slm, service, ui]
acceptance_criteria:
  - "studyfrog-report identifies leech items using multi-signal criteria (total lapses, recent lapse streak, cumulative review time share, and low stability-to-reviews ratio)"
  - "The analytics engine detects interfering item pairs that are frequently failed in close temporal proximity and share high lexical/semantic similarity, tags, or wikilinks"
  - "The Reports screen (and stack Reports tab) lists leeches and confusion pairs with direct actions to open, split/rewrite, pause, or launch a contrastive drill"
---

## Context
In spaced repetition, a small fraction of cards ("leeches") often consumes a disproportionate share of total study time because the prompt is ambiguous, overloaded, or suffers from **retroactive/proactive interference** with a similar card in the vault. Traditional SRS tools only count raw `Again` lapses (e.g., 8 lapses). Because StudyFrog also tracks per-item response latency, difficulty marks, familiarity, `[[Wikilinks]]`, tags, and semantic/token similarity, it can diagnose *why* an item is failing—distinguishing an isolated overloaded card from an interfering pair of look-alike concepts.

## Description
Build multi-signal leech diagnostics and interference cluster detection:
1. **Multi-Signal Leech Scoring (`crates/studyfrog-report/src/leech.rs`):**
   - Evaluate each scheduled item across four signals:
     - `lapses: u32` (number of `Again` ratings after the first review, default threshold ≥ 4).
     - `consecutive_hard_or_again: u32` (recent struggle streak ≥ 3).
     - `time_burden_ratio: f32` (item's cumulative review time divided by the stack's median per-item cumulative review time).
     - `efficiency_ratio: f32` (`stability / reviews`, flagging items reviewed ≥ 6 times whose stability remains under 7 days).
   - Produce a ranked `LeechItem` list with a diagnostic reason (`RepeatedLapses`, `HighLatencyBurden`, `StagnantStability`).
2. **Interference & Confusion Pair Detection:**
   - For items with ≥ 2 lapses, check other items in the same stack (or linked via `[[Wikilinks]]` / shared tags) that have high prompt/answer token overlap or embedding similarity (`studyfrog-slm`) and co-occurring lapses or slow responses.
   - Surface ranked `ConfusionPair { item_a, item_b, similarity, co_lapses, shared_context }` records.
3. **UI Actions in Reports & Stack View:**
   - Add a **Leeches & Interference** section in Reports (and the Stack `Reports` tab) with one-click buttons to `Edit / Rewrite`, `Pause item`, or `Compare side-by-side`.

## Acceptance Criteria
- [x] studyfrog-report identifies leech items using multi-signal criteria (total lapses, recent lapse streak, cumulative review time share, and low stability-to-reviews ratio)
- [x] The analytics engine detects interfering item pairs that are frequently failed in close temporal proximity and share high lexical/semantic similarity, tags, or wikilinks
- [x] The Reports screen (and stack Reports tab) lists leeches and confusion pairs with direct actions to open, split/rewrite, pause, or launch a contrastive drill

## Design Notes
- Touchpoints:
  - `crates/studyfrog-report/src/leech.rs`: pure leech and confusion-pair detection functions.
  - `crates/studyfrog-service/src/reports.rs`: gather item states, review histories, latencies, and token similarity into the report bundle.
  - `crates/studyfrog-ui/src/view/reports.rs`: render actionable leech and interference cards.
- Strictly follow P1 (free functions only) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and verified:
- `crates/studyfrog-report/src/leech.rs`: four signals per item (lapses after the first review >= 4, Hard/Again
  streak at the end >= 3, time spent over the stack's median time per item >= 3, >= 6 reviews and stability under
  7 days), every applying reason strongest first, ranked by the sum of each signal over its threshold.
- `crates/studyfrog-report/src/confusion.rs`: for items with >= 2 lapse days, pairs whose lapses fall within a day
  of each other at least twice (counted both ways) and that share >= 30 % of their words (hashed, Jaccard), a tag,
  or a wikilink; ranked. Semantic (embedding) similarity is not used: the lexical overlap, tags and links meet the
  criterion; recorded here as a choice (no model is needed and it runs in 3 ms for 400 items).
- Tests with hand-checked fixtures and property tests: `crates/studyfrog-report/tests/leech.rs`.
- Service `crates/studyfrog-service/src/analytics_leech.rs` (titles, stacks, tags and links from the index, words
  from the item files, time from the run log, stability from the primary stack's state) -> `analytics.leeches` and
  `analytics.pairs`; `crates/studyfrog-service/src/drill.rs` starts a practice run of exactly the given items (in
  that order, a forgotten one comes back, the schedule is untouched). Tested on a fixture vault:
  `crates/studyfrog-service/tests/analytics_leeches.rs`.
- Reports > Leeches and the stack's Reports tab (filtered to the stack and its sub-stacks): each leech with its
  reasons and numbers and "Edit or split" (opens the item page), "Pause" (status paused), "Drill"; each pair with
  why, links to both items and "Drill ... together" (a contrastive drill). Harness tests
  `crates/studyfrog-ui/tests/reports_leeches_ui.rs`; screenshot `--screen reports:leeches` on a demo vault.
- Benchmarks: leeches of 16,000 items 0.7 ms; confusion pairs of 400 lapsing items 3.0 ms.

"Split" has no automatic splitter: the button opens the item page to rewrite or split it by hand.

## Dependencies
EPIC-034.

## Open Questions
None.
