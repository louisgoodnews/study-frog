---
id: STORY-0105
type: story
title: FSRS state-simulation forecasting, study-minute projection, and exam deadline pacer
status: in_progress
epic: EPIC-034
parents: [EPIC-034]
estimate: 8
labels: [m12, report, srs, service, ui]
acceptance_criteria:
  - "The forecast engine simulates each item's actual FSRS-6/SM-2 state forward over 30, 90, or 365 days with P10/P50/P90 confidence bands and stage breakdowns (New, Learning, Young, Mature)"
  - "Daily review projections are converted into expected study minutes using empirical per-stack answer and reading latencies"
  - "Stacks with an exam or due date display an Exam Deadline Pacer calculating the exact daily new-item and review quota required to reach target readiness before the deadline"
---

## Context
StudyFrog's existing forecast (`crates/studyfrog-report/src/forecast.rs`) projects future load by multiplying each item's current interval by a single scalar `growth` factor and adding `lapse_rate` on the next day. As documented in `docs/specs/reports.md`, this simplified model:
1. Ignores each item's actual FSRS-6 `MemState { stability, difficulty }` and familiarity factor.
2. Assumes 0% missed/postponed days, causing an 8–11% systematic bias when learners occasionally skip a day.
3. Only outputs raw *review counts* rather than expected **study minutes** (whereas a stack of complex math questions takes 5× longer per review than vocabulary cards).
4. Does not calculate a forward **Exam Deadline Pacer** telling the student how many new cards and reviews per day are needed to clear a stack before an exam date.

## Description
Upgrade StudyFrog's forecasting engine to a full FSRS/SM-2 state-transition simulator with time-budget projection and deadline pacing:
1. **State-Transition & Monte Carlo Workload Forecast (`crates/studyfrog-report/src/forecast_sim.rs`):**
   - Simulate each item's actual `MemState` forward day-by-day over a configurable horizon (`30`, `90`, or `365` days) using deterministic seeded rollouts (`N` trajectories, e.g., `N = 16`) that incorporate the user's empirical rating distribution (`Again`, `Hard`, `Good`, `Easy` conditioned on retrievability $R$) and empirical daily adherence rate (`miss_probability`).
   - Output per-day:
     - Stage breakdown: `new_reviews`, `relearning_reviews`, `young_reviews` ($S < 21\text{d}$), `mature_reviews` ($S \ge 21\text{d}$).
     - Uncertainty band: `p10_reviews`, `p50_reviews` (expected), `p90_reviews`.
     - `expected_minutes: f32` computed from the stack's median/mean active time per new, relearning, and review card.
2. **Exam Deadline Pacer:**
   - For any stack with a configured `due` / `exam` date (or user-selected target date), compute:
     - `unseen_items: usize` and `volatile_items: usize` (items with predicted retrievability $R(\text{exam\_day}) < r_{\text{target}}$).
     - `required_new_per_day: u32` and `projected_daily_minutes: f32` needed so that ≥ 90% of items in the stack reach target retrievability on the exam date, leaving a configurable final review buffer (e.g., last 3 days reserved for consolidation rather than unseen items).
3. **UI Integration in Reports, Calendar & Stack Overview:**
   - Allow toggling the Forecast chart between **Review count** and **Study minutes**, and between `30d`, `90d`, and `365d` horizons with stacked stage bars (`New`, `Learning`, `Young`, `Mature`).
   - Show the **Exam Pacer** card on the Stack `Summary` and `Schedule` tabs whenever a stack has a deadline.

## Acceptance Criteria
- [ ] The forecast engine simulates each item's actual FSRS-6/SM-2 state forward over 30, 90, or 365 days with P10/P50/P90 confidence bands and stage breakdowns (New, Learning, Young, Mature)
- [ ] Daily review projections are converted into expected study minutes using empirical per-stack answer and reading latencies
- [ ] Stacks with an exam or due date display an Exam Deadline Pacer calculating the exact daily new-item and review quota required to reach target readiness before the deadline

## Design Notes
- Keep `forecast.rs` or add `forecast_sim.rs` and `exam_pacer.rs` in `crates/studyfrog-report/src/`.
- Ensure simulation of 5,000 items over 90 days completes well under the 50 ms budget by using lightweight fixed-seed PRNG (`next_random`) and pre-binned items for long horizons.
- Enforce P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-034.

## Open Questions
None.
