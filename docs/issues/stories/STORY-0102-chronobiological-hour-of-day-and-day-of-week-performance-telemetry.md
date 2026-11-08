---
id: STORY-0102
type: story
title: Chronobiological hour-of-day and day-of-week performance telemetry
status: in_progress
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
- [ ] studyfrog-report computes recall rate, review volume, and median/p90 answer latency across the 24 local hours of the day and 7 days of the week
- [ ] The Reports screen displays a chronobiological performance profile highlighting the learner's biological peak study window and high-latency/high-lapse troughs
- [ ] Calculations respect the user's local UTC offset and require at least a configurable minimum sample threshold per bucket before surfacing peak/trough recommendations

## Design Notes
- Touchpoints:
  - `crates/studyfrog-report/src/chronotype.rs`: pure aggregation functions and structs (`HourBucket`, `WeekdayBucket`, `ChronotypeReport`).
  - `crates/studyfrog-service/src/reports.rs`: wire review timestamps and event latencies into `chronotype_report`.
  - `crates/studyfrog-ui/src/view/reports.rs`: render circadian and weekday visual breakdowns.
- Comply with P1 (zero inherent `impl` blocks) and P2 (≤ 250 lines per file).

## Dependencies
EPIC-034.

## Open Questions
None.
