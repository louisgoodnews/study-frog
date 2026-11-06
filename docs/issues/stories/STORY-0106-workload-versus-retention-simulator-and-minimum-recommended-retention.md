---
id: STORY-0106
type: story
title: Workload-versus-retention simulator and minimum recommended retention (CMRR)
status: backlog
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
- [ ] studyfrog-report simulates long-term daily study workload (reviews/day and minutes/day) and total memorized items across desired retention targets from 0.75 to 0.97
- [ ] The simulator computes the learner's Minimum Recommended Retention (CMRR) that maximizes retained knowledge per minute of study time and identifies the point where lower retention increases relearning workload
- [ ] Users can inspect the Workload vs. Retention curve in Reports and Stack Settings and apply the recommended retention with one click

## Design Notes
- Touchpoints:
  - `crates/studyfrog-report/src/cmrr.rs`: pure simulation across retention grid.
  - `crates/studyfrog-service/src/reports.rs`: compute `CmrrReport` for vault and stack scopes.
  - `crates/studyfrog-ui/src/view/reports.rs` and `stack_view`: render trade-off chart and one-click apply action.
- Adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-034, STORY-0105.

## Open Questions
None.
