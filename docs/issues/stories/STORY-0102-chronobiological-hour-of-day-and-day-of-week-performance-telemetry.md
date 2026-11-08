---
id: STORY-0102
type: story
title: Chronobiological hour-of-day and day-of-week performance telemetry
status: done
epic: EPIC-034
parents: [EPIC-034]
estimate: 3
labels: [m12, report, service, ui]
acceptance_criteria:
  - "studyfrog-report computes recall rate, review volume, and median/p90 answer latency across the 24 local hours of the day and 7 days of the week"
  - "The Reports screen displays a chronobiological performance profile highlighting the learner's biological peak study window and high-latency/high-lapse troughs"
  - "Calculations respect the user's local UTC offset and require at least a configurable minimum sample threshold per bucket before surfacing peak/trough recommendations"
---

## Context
Cognitive science research demonstrates that working memory capacity, retrieval speed, and synaptic consolidation fluctuate significantly across the 24-hour circadian cycle and across weekdays versus weekends. StudyFrog already records the exact Unix millisecond timestamp (`at_ms`), local UTC offset (`offset_minutes`), rating (`Again`, `Hard`, `Good`, `Easy`), and reveal latency (`ms`) for every review, yet the Reports screen only aggregates retention by elapsed-day gap and overall latency percentiles. Surfacing a chronobiological profile helps learners schedule demanding stacks during their empirical peak cognitive hours.

## Description
Implement circadian (hour-of-day) and weekly (day-of-week) performance and latency analytics:
1. **Pure Chronobiological Aggregation (`crates/studyfrog-report/src/chronotype.rs`):**
   - Group historical reviews and `FlipSample` latency measurements into 24 local-hour buckets (`0..24`) and 7 weekday buckets (`Mon..Sun`) using `(at_ms + offset_minutes * 60_000)`.
   - For each bucket, compute:
     - `reviews: u32` and `recalled: u32` (retention rate `recalled / reviews`).
     - `easy_good_rate: f32` (share of reviews rated `Good` or `Easy`).
     - `median_ms: Option<u32>` and `p90_ms: Option<u32>` (response latency percentiles).
   - Identify the learner's **Peak Window** (e.g., the contiguous 2–3 hour window with at least `MIN_BUCKET_REVIEWS` that maximizes `retention_rate / log(median_ms)`) and **Fatigue/Trough Window**.
2. **Service & UI Presentation (`crates/studyfrog-service/src/reports.rs`, `crates/studyfrog-ui/src/view/reports.rs`):**
   - Include `ChronotypeReport` in `ReportBundle` (and stack-scoped reports).
   - Render a 24-hour bar/heatmap chart and a 7-day summary table in the Reports view with an actionable insight banner (e.g., *"Your recall is highest (94%, median 3.1 s) between 09:00 and 11:00"*).

## Acceptance Criteria
- [x] studyfrog-report computes recall rate, review volume, and median/p90 answer latency across the 24 local hours of the day and 7 days of the week
- [x] The Reports screen displays a chronobiological performance profile highlighting the learner's biological peak study window and high-latency/high-lapse troughs
- [x] Calculations respect the user's local UTC offset and require at least a configurable minimum sample threshold per bucket before surfacing peak/trough recommendations

## Design Notes
- Touchpoints:
  - `crates/studyfrog-report/src/chronotype.rs`: pure aggregation functions and structs (`HourBucket`, `WeekdayBucket`, `ChronotypeReport`).
  - `crates/studyfrog-service/src/reports.rs`: wire review timestamps and event latencies into `chronotype_report`.
  - `crates/studyfrog-ui/src/view/reports.rs`: render circadian and weekday visual breakdowns.
- Comply with P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Progress (2026-11-08)
Built and verified:
- `crates/studyfrog-report/src/chronotype.rs`: 24 local-hour and 7 weekday buckets (reviews, recalled, Good/Easy,
  answers, median and p90 answer time); each review and answer time carries the UTC offset of its own instant;
  peak and trough are the 2-hour windows (wrapping midnight, not overlapping) that maximise / minimise
  `retention / ln(median ms)` among windows whose hours all have at least the threshold of reviews; best and worst
  weekday likewise. `with_windows` recomputes them for another threshold. The window median is the answer-weighted
  mean of the hourly medians (documented). Tests: `crates/studyfrog-report/tests/chronotype.rs` (offsets east and
  west, peak 09-11 / trough 20-22 on a hand-made fixture, threshold withholding, percentiles, property test).
- `crates/studyfrog-report/src/session_log.rs`: every rated item of a run with its position, active time, reveal
  latency and time to rate (pauses excluded); answer times are placed in wall time by the run's start.
- Service: `ReportBundle.analytics.chronotype`; `build_reports_local` asks the system time zone for the offset of
  each hour of history (cached per hour), `build_reports_with` takes any offset function. Tests on a fixture vault
  with a real run log: `crates/studyfrog-service/tests/analytics_habits.rs` (UTC, +11 h, -15 h).
- Reports > Habits: insight lines for the peak and the low point (or why there is none), a threshold choice
  (5/10/20/50 reviews per hour), hour and weekday rows with the recall on a bar and every other number beside it
  (`view/reports_habits.rs`, `view/report_rows.rs`). Harness tests in `tests/reports_analytics_ui.rs`; screenshot
  `--screen reports:habits` looked at on a demo vault.
- Benchmark: chronotype of 103,596 reviews and answer times 3.3 ms (budget 50 ms).

## Dependencies
EPIC-034.

## Open Questions
None.
