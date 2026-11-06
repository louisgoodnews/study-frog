---
id: STORY-0103
type: story
title: Within-session cognitive fatigue and pacing decay analysis
status: backlog
epic: EPIC-034
parents: [EPIC-034]
estimate: 3
labels: [m12, report, rehearsal, service, ui]
acceptance_criteria:
  - "studyfrog-report analyzes rehearsal run event logs by elapsed session duration (5-minute buckets) and item sequence position to measure accuracy decay and response latency inflation"
  - "The Reports screen visualizes within-session pacing and accuracy curves and recommends an optimal rehearsal session duration before cognitive fatigue sets in"
  - "Active rehearsal runs optionally display a subtle non-blocking notice when real-time latency spikes significantly above the user's session baseline"
---

## Context
During long rehearsal sessions, cognitive exhaustion causes a predictable pattern: response latency gradually climbs while recall accuracy (and written-answer similarity scores) begins to drop. Because StudyFrog logs every `RunEvent` (`ItemShown`, `ItemFlipped`, `AnswerSubmitted`, `ItemRated`, `RunPaused`, `RunResumed`) with pause-adjusted monotonic timestamps (`t_ms`), it can pinpoint the exact duration or item count at which a learner's efficiency degrades.

## Description
Implement within-session cognitive fatigue and pacing decay telemetry:
1. **Pure Session-Decay Analytics (`crates/studyfrog-report/src/fatigue.rs`):**
   - Reconstruct active elapsed time per item within each rehearsal run (excluding paused intervals).
   - Group in-run reviews by **Elapsed Session Time Buckets** (`0–5 min`, `5–10 min`, `10–15 min`, `15–20 min`, `20–30 min`, `30+ min`) and by **Item Position Buckets** (`1–10`, `11–20`, `21–35`, `36–50`, `51+`).
   - Compute per-bucket recall rate (`rating != Rating::Again`), `Good/Easy` share, median reveal latency (`p50_ms`), and tail latency (`p90_ms`).
   - Detect the **Fatigue Inflection Point**: the bucket where recall drops by ≥ 8 percentage points or median latency increases by ≥ 35% relative to the first 10 minutes of the user's sessions.
2. **Reports & Rehearsal Integration:**
   - Display the within-session fatigue curve and **Recommended Session Cap** (e.g., *"Your accuracy holds steady for ~18 minutes (~28 items) before latency rises 42%"*) in the Reports screen.
   - Optionally use the recommended cap to pre-populate countdown timer presets in Rehearsal setup.

## Acceptance Criteria
- [ ] studyfrog-report analyzes rehearsal run event logs by elapsed session duration (5-minute buckets) and item sequence position to measure accuracy decay and response latency inflation
- [ ] The Reports screen visualizes within-session pacing and accuracy curves and recommends an optimal rehearsal session duration before cognitive fatigue sets in
- [ ] Active rehearsal runs optionally display a subtle non-blocking notice when real-time latency spikes significantly above the user's session baseline

## Design Notes
- Touchpoints:
  - `crates/studyfrog-report/src/fatigue.rs`: pure bucket aggregation and inflection-point detection.
  - `crates/studyfrog-service/src/reports.rs`: feed `load_all_events` and review ratings into `fatigue_report`.
  - `crates/studyfrog-ui/src/view/reports.rs`: render session duration vs. accuracy/latency chart.
- Keep all files ≤ 250 lines (P2) and free of inherent `impl` blocks (P1).

## Dependencies
EPIC-034.

## Open Questions
None.
