---
id: STORY-0106
type: story
title: "Workload-versus-retention simulator and minimum recommended retention (CMRR)"
status: done
epic: EPIC-034
parents: [EPIC-034]
estimate: 5
labels: [m12, report, srs, service, ui]
acceptance_criteria:
  - "studyfrog-report simulates long-term daily study workload (reviews/day and minutes/day) and total memorized items across desired retention targets from 0.75 to 0.97"
  - "The simulator computes the learner's Minimum Recommended Retention (CMRR) that maximizes retained knowledge per minute of study time and identifies the point where lower retention increases relearning workload"
  - "Users can inspect the Workload vs. Retention curve in Reports and Stack Settings and apply the recommended retention with one click"
---

## Context
In FSRS-6, choosing a **Desired Retention** ($r \in [0.75, 0.97]$) governs the fundamental trade-off between daily study workload and memory retention:
- Raising target retention from `0.90` to `0.95` nearly doubles daily review workload for a 5% gain in recall.
- Conversely, lowering target retention *too* far (e.g., below `0.80`) causes so many forgetting lapses (`Rating::Again`) that relearning reviews actually **increase** total daily workload while retention plummets.

In modern FSRS ecosystems, **Compute Minimum Recommended Retention (CMRR)** solves this by simulating the learner's deck across candidate retention targets and plotting both **Daily Workload (minutes/day)** and **Efficiency (Memorized Items per Daily Study Minute)**.

## Description
Implement a pure Rust Workload-vs-Retention & CMRR simulator in `studyfrog-report` and surface it in Reports and Stack Settings:
1. **Pure CMRR & Trade-Off Curve Simulation (`crates/studyfrog-report/src/cmrr.rs`):**
   - Sweep candidate desired retention targets `r` in `[0.75, 0.78, 0.80, 0.82, 0.84, 0.86, 0.88, 0.90, 0.92, 0.94, 0.96, 0.97]` over a 180-day or 365-day simulation window using the stack's/vault's FSRS parameters, current item states, and empirical rating latencies (`ms_per_review` vs. `ms_per_lapse`).
   - For each candidate `r`, compute:
     - `daily_reviews: f32` and `daily_minutes: f32`.
     - `mean_retention: f32` and `expected_memorized_items: f32` ($\sum_i R_i$).
     - `efficiency: f32` (`expected_memorized_items / daily_minutes.max(0.1)`).
   - Identify:
     - `min_workload_retention: f32` (the retention below which lapses cause workload to rise again).
     - `optimal_efficiency_retention: f32` (the CMRR sweet spot maximizing `efficiency`).
2. **Reports & Stack Settings Integration:**
   - Render the **Workload vs. Desired Retention** curve in Reports and in the Stack `Settings` / `Schedule` view.
   - Provide an `"Apply recommended retention"` button that updates `desired_retention` for the vault or active stack.

## Acceptance Criteria
- [x] studyfrog-report simulates long-term daily study workload (reviews/day and minutes/day) and total memorized items across desired retention targets from 0.75 to 0.97
- [x] The simulator computes the learner's Minimum Recommended Retention (CMRR) that maximizes retained knowledge per minute of study time and identifies the point where lower retention increases relearning workload
- [x] Users can inspect the Workload vs. Retention curve in Reports and Stack Settings and apply the recommended retention with one click

## Design Notes
- Touchpoints:
  - `crates/studyfrog-report/src/cmrr.rs`: pure simulation across retention grid.
  - `crates/studyfrog-service/src/reports.rs`: compute `CmrrReport` for vault and stack scopes.
  - `crates/studyfrog-ui/src/view/reports.rs` and `stack_view`: render trade-off chart and one-click apply action.
- Adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and verified:
- `crates/studyfrog-report/src/cmrr.rs`: the deck simulated (STORY-0105 engine, same random numbers at every
  target) for 180 days at each of 0.75, 0.78, ..., 0.97 with FSRS parameters at that retention: reviews and minutes
  a day, cards remembered at the end (sum of recall), mean recall, cards per daily minute; the least-work target
  (below it forgetting adds more relearning than it saves) and the recommendation (most cards per minute at or above
  the least-work target). Tests: `crates/studyfrog-report/tests/cmrr.rs` (grid, cost rises at the top, the
  recommendation rule, a U-shaped curve when relearning is costly and intervals are capped, empty deck, seeds).
- `crates/studyfrog-config/src/retention_edit.rs`: sets `request_retention` in `_stack.toml` (top level) or
  `studyfrog.toml` (`[srs]`) keeping comments and other keys (tests `crates/studyfrog-config/tests/retention_edit.rs`).
- Service `crates/studyfrog-service/src/analytics_cmrr.rs`: the vault's curve (4 futures) and each stack's own
  cards (2 futures) in `analytics.plan.cmrr`; `apply_retention` writes the file, reindexes and returns the vault
  (fixture-vault tests `crates/studyfrog-service/tests/analytics_cmrr.rs`).
- UI: the curve (target with marks "recommended", "least work", "now"; minutes a day on the bar; reviews, cards
  remembered and cards per minute as text) and "Apply recommended retention X to ..." on Reports > Workload (vault)
  and on the stack's Reports tab (that stack's `_stack.toml`); the result refreshes the vault and rebuilds the
  reports (`view/report_cmrr.rs`; harness and worker tests `crates/studyfrog-ui/tests/reports_cmrr_ui.rs`;
  screenshot `--screen reports:workload`).
- Benchmark: CMRR of 5,000 cards, 12 targets, 180 days, 4 futures: 45.4 ms (budget 50 ms); the per-stack curves
  use 2 futures (about half of that together).

Note on criterion 3: the stack page has no "Settings" tab (a stack's settings are its `_stack.toml`); the stack's
curve and its one-click apply are on the stack's Reports tab and write that file.

## Dependencies
EPIC-034, STORY-0105.

## Open Questions
None.
