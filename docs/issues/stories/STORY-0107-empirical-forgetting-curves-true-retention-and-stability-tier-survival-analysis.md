---
id: STORY-0107
type: story
title: Empirical forgetting curves, true retention, and stability-tier survival analysis
status: backlog
epic: EPIC-034
parents: [EPIC-034]
estimate: 3
labels: [m12, report, srs, service, ui]
acceptance_criteria:
  - "studyfrog-report computes empirical forgetting curves grouped by FSRS stability tier (<7d, 7-30d, 30-90d, 90d+) and compares them against theoretical retrievability R(t, S)"
  - "The analytics engine calculates True Retention (retention on mature items reviewed within ±20% of their scheduled due interval) separately from early and overdue reviews"
  - "The Reports Retention tab displays True Retention, Young vs. Mature retention, and empirical vs. predicted forgetting curves with estimated personal memory half-life"
---

## Context
Currently, `crates/studyfrog-report/src/retention.rs` groups all reviews solely by elapsed days (`1`, `2`, `3–4`, `5–7`, `8–14`, `15–30`, `31–90`, `91+`), mixing together brand-new cards ($S = 2\text{d}$) that are 14 days overdue with mature cards ($S = 60\text{d}$) reviewed early at 14 days. In spaced-repetition analytics, two clearer metrics are essential:
1. **True Retention:** Recall rate specifically on **mature** items ($S \ge 21\text{d}$) reviewed when they are actually due (relative elapsed ratio $t / I \in [0.8, 1.2]$), plus separate breakdowns for **Young Retention** ($S < 21\text{d}$) and **Overdue Retention** ($t / I > 1.2$).
2. **Stability-Tier Empirical Forgetting Curves:** Grouping reviews by the item's stability tier at the time of review (`S < 7d`, `7–30d`, `30–90d`, `90d+`) and plotting empirical recall against relative elapsed time ($t / S$) alongside FSRS's theoretical power-law forgetting curve $R(t, S) = (1 + \text{factor} \cdot t / S)^{-w_{20}}$.

## Description
Enhance StudyFrog's retention analytics in `studyfrog-report`, `studyfrog-service`, and `studyfrog-ui`:
1. **True Retention & Timeliness Breakdown (`crates/studyfrog-report/src/retention_survival.rs`):**
   - Replay FSRS states across the review history to know `(stability, scheduled_interval, elapsed_days)` at each review event.
   - Compute:
     - `true_retention: Option<f32>` (mature items $S \ge 21\text{d}$ reviewed at $0.8 \le t / I \le 1.2$).
     - `young_retention: Option<f32>` ($S < 21\text{d}$).
     - `mature_retention: Option<f32>` ($S \ge 21\text{d}$, all timeliness).
     - `overdue_retention: Option<f32>` ($t / I > 1.2$, compared against FSRS predicted $R(t, S)$).
2. **Stability-Tier Forgetting Curves:**
   - Partition reviews into 4 stability tiers (`<7d`, `7–30d`, `30–90d`, `90d+`) and bin by elapsed days or relative elapsed ratio ($t / S$).
   - Compute both empirical recall share and mean FSRS predicted retrievability $R(t, S)$ per bin, plus Root Mean Squared Calibration Error (RMSE).
3. **UI Presentation in Reports:**
   - Upgrade the Retention section of the Reports screen with summary KPI pills (`True Retention`, `Young Retention`, `Mature Retention`, `Overdue Resilience`) and stability-tier forgetting curve comparisons.

## Acceptance Criteria
- [ ] studyfrog-report computes empirical forgetting curves grouped by FSRS stability tier (<7d, 7-30d, 30-90d, 90d+) and compares them against theoretical retrievability R(t, S)
- [ ] The analytics engine calculates True Retention (retention on mature items reviewed within ±20% of their scheduled due interval) separately from early and overdue reviews
- [ ] The Reports Retention tab displays True Retention, Young vs. Mature retention, and empirical vs. predicted forgetting curves with estimated personal memory half-life

## Design Notes
- Touchpoints:
  - `crates/studyfrog-report/src/retention_survival.rs`: pure survival and true-retention calculations.
  - `crates/studyfrog-service/src/reports.rs`: integrate `SurvivalReport` into `ReportBundle`.
  - `crates/studyfrog-ui/src/view/reports.rs`: render KPI cards and stability-tier curves.
- Adhere to P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-034.

## Open Questions
None.
