---
id: EPIC-034
type: epic
title: Behavioral learning analytics, FSRS optimizer, and workload forecasting
status: done
parents: []
labels: [m12, report, srs, service, ui]
acceptance_criteria:
  - "StudyFrog analyzes chronobiological (hour-of-day and day-of-week) performance, within-session cognitive fatigue, empirical forgetting curves, true retention, and leech/interference clusters from existing review and event logs"
  - "The forecasting engine simulates per-item FSRS-6/SM-2 state transitions forward in time to project daily review counts (by learning stage), expected study minutes, confidence bands, and exam-deadline daily quotas"
  - "Users can run a Workload vs. Desired Retention (CMRR) simulation to find the optimal retention target that maximizes knowledge retained per minute of study time"
---

## Context
StudyFrog's current reporting and forecasting engine (`crates/studyfrog-report`) provides an 8-bucket retention table, overall answer-time percentiles, a simplified geometric review-count forecast (`interval * growth`), a 28-day backtest, familiarity replay, and diary self-assessment calibration.

A project analysis of `studyfrog-report` combined with research into modern spaced-repetition telemetry (FSRS-6 workload simulation, Compute Minimum Recommended Retention / CMRR, chronobiological memory research, response-latency fatigue modeling, and retroactive/proactive interference detection) reveals two major opportunities:
1. **Rich Behavioral & Cognitive Telemetry Already Captured in SQLite:** StudyFrog's `review` and `run_event` tables already record millisecond timestamps (`at_ms`), local UTC offsets, per-item monotonic reveal latencies (`t_ms`), difficulty marks, and written answer scores—yet the app does not analyze *when* the learner remembers best (circadian profile), *when* fatigue sets in during a session, *which* items are leeches or mutually interfering pairs, or how empirical recall compares against theoretical FSRS forgetting curves across stability tiers.
2. **State-Driven Forecasting, Time Budgeting & Exam Pacing:** The current forecast in `crates/studyfrog-report/src/forecast.rs` multiplies intervals by a single scalar median `growth` and applies a flat `lapse_rate`, ignoring individual item `MemState { stability, difficulty }`, review latency (study *minutes* vs. raw review counts), and exam deadlines. Upgrading to an FSRS state-transition simulator enables time-budget forecasting, exam-readiness daily quotas, and CMRR (Workload vs. Retention optimization).

## Description
Expand StudyFrog's statistical analysis of learning patterns/behaviors and its forecasting system across six cohesive stories:
1. **Chronobiological (Hour-of-Day & Day-of-Week) Performance Telemetry (`STORY-0102`):**
   - Aggregate recall rate, median/p90 response latency, and review volume across the 24-hour local clock and 7 days of the week to identify the learner's biological peak study window and circadian troughs.
2. **Within-Session Cognitive Fatigue & Pacing Decay Analysis (`STORY-0103`):**
   - Analyze `run_event` logs by elapsed session duration (5-minute buckets) and item sequence position to detect where response latency spikes and recall accuracy drops, surfacing a personalized recommended session length.
3. **Leech Diagnostics, Interference Detection & Confusion Clusters (`STORY-0104`):**
   - Automatically flag **leeches** (items with repeated lapses, high cumulative review time, or low stability-to-review ratios) and detect **interfering item pairs** (items sharing links/tags or high semantic similarity that frequently lapse in the same window).
4. **FSRS State-Simulation Forecasting, Study-Minute Projection & Exam Deadline Pacer (`STORY-0105`):**
   - Simulate each item's actual FSRS-6/SM-2 state forward over 30–365 days with postponement awareness, breaking projected daily workload into `New`, `Learning/Relearning`, `Young` (<21d), and `Mature` (≥21d) stages, converting review counts into expected **study minutes** using per-stack latency distributions, and computing daily new/review quotas to hit target readiness before a stack's exam date.
5. **Workload vs. Retention Simulator & Minimum Recommended Retention — CMRR (`STORY-0106`):**
   - Simulate long-term workload across desired retention targets (`0.75` to `0.97`) using the learner's history to plot the **Workload vs. Retention** trade-off curve and identify the **Minimum Recommended Retention** (the inflection point maximizing knowledge retained per unit of study time).
6. **Empirical Forgetting Curves, True Retention & Stability-Tier Survival Analysis (`STORY-0107`):**
   - Plot empirical recall rates against FSRS theoretical retrievability $R(t, S)$ across stability tiers (`<7d`, `7–30d`, `30–90d`, `90d+`) and compute **True Retention** (retention on mature items reviewed on time) vs. early and overdue retention.

## Acceptance Criteria
- [x] StudyFrog analyzes chronobiological (hour-of-day and day-of-week) performance, within-session cognitive fatigue, empirical forgetting curves, true retention, and leech/interference clusters from existing review and event logs
- [x] The forecasting engine simulates per-item FSRS-6/SM-2 state transitions forward in time to project daily review counts (by learning stage), expected study minutes, confidence bands, and exam-deadline daily quotas
- [x] Users can run a Workload vs. Desired Retention (CMRR) simulation to find the optimal retention target that maximizes knowledge retained per minute of study time

## Design Notes
Child stories:
- STORY-0102: Chronobiological hour-of-day and day-of-week performance telemetry (estimate: 3)
- STORY-0103: Within-session cognitive fatigue and pacing decay analysis (estimate: 3)
- STORY-0104: Leech diagnostics, interference detection, and confusion clusters (estimate: 5)
- STORY-0105: FSRS state-simulation forecasting, study-minute projection, and exam deadline pacer (estimate: 8)
- STORY-0106: Workload-versus-retention simulator and minimum recommended retention (CMRR) (estimate: 5)
- STORY-0107: Empirical forgetting curves, true retention, and stability-tier survival analysis (estimate: 3)

Architecture and performance guardrails:
- All statistical and simulation functions in `crates/studyfrog-report` must remain 100% pure, deterministic (seeded RNG for Monte Carlo simulation), and free of I/O (`check-deps.sh`).
- Keep per-report computation within the 50 ms budget on 100,000 reviews (`docs/specs/reports.md`).
- Enforce zero inherent `impl` blocks (P1) and ≤ 250 lines per `.rs` file (P2).

## Progress (2026-11-08)
All six stories are done (each with its own Progress section): STORY-0107 true retention and forgetting curves
(Reports > Memory), STORY-0102 hour/weekday profile and STORY-0103 fatigue (Reports > Habits, slow-answer line in
runs), STORY-0104 leeches and confusion pairs with drills (Reports > Leeches, stack Reports tab), STORY-0105 the
simulated forecast with study minutes and the exam pacer (Reports > Workload, stack Summary and Reports), STORY-0106
workload against desired retention with one-click apply. Pure functions in `studyfrog-report` (seeded simulations),
assembled in `studyfrog-service/src/analytics*.rs` into `ReportBundle.analytics`; benchmarks in
`crates/studyfrog-report/benches/analytics.rs`, all within 50 ms (docs/contributing/BENCHMARKING.md). Checked by
unit, property, fixture-vault and harness tests and by screenshots (`--screen reports:<tab>`); not yet used by a
person in a real window.

## Dependencies
EPIC-004 (SRS Scheduling), EPIC-015 (Reporting and Forecasting), EPIC-019 (Rehearsal Insights), and EPIC-024 (Diary and Calibration).

## Open Questions
None.
