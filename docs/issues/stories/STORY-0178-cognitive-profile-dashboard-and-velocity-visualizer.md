---
id: STORY-0178
type: story
title: "Cognitive Profile Dashboard and Learning Velocity Visualization in Reports"
status: review
epic: EPIC-060
parents: ['EPIC-060']
estimate: 3
labels: [m14, ui, report]
acceptance_criteria:
  - 'Cognitive Profile tab added to the Reports screen'
  - 'Visual charts showing personal learning velocity, response latency distribution, and semantic accuracy trends'
  - 'Metacognitive calibration widget comparing subjective user ratings against objective telemetry metrics'
---

## Context
Learners benefit from transparent insight into how their memory actually operates: whether they over-rate their mastery, how fast they recall different subjects, and how their cognitive velocity evolves across semesters.

## Description
Build the Cognitive Profile visualization in `studyfrog-ui`:
- Add a new tab/subview under Reports displaying the learner's cognitive profile summary.
- Render charts using `egui` painter primitives and palette tokens (`palette_of(ui)`):
  - Learning velocity curve (stability growth per review minute).
  - Latency vs text complexity scatter/distribution.
  - Semantic accuracy progression over time.
- Display a Metacognitive Calibration score highlighting discrepancies between subjective ratings ("Easy") and objective latencies.

## Acceptance Criteria
- [x] Cognitive Profile tab added to the Reports screen
- [ ] Visual charts showing personal learning velocity, response latency distribution, and semantic accuracy trends
- [x] Metacognitive calibration widget comparing subjective user ratings against objective telemetry metrics

## Implementation (2026-11-21)
The Reports screen has a "Cognitive profile" tab. It is built from the review history (`studyfrog-service::cognitive_report`, a pure function over `HistoryRow`s) and is part of `ReportBundle`.

Deviations from the description, because the data is not stored: the app records no telemetry (`studyfrog-store::telemetry` has no caller, so the store is never written), and reviews keep no semantic score or stability. The profile therefore uses what the history holds:
- Latency against length: answer time against the expected time from the word count (scatter, latest 500 answers).
- Learning velocity: passing answers (Good or Easy) per minute of answering, the slope over cumulative answer time. It is a proxy for stability growth per minute.
- Recall trend: the share of passing answers in each week, in place of a semantic accuracy trend.
- Calibration: the share of timed answers rated Easy against the share answered within the expected time, with a note when they differ by more than 15 points.

Telemetry is now recorded on every timed rating (STORY-0176). The profile still reads the review history; semantic scores are not produced yet, so they are not shown.

Tests: `studyfrog-service/tests/cognitive_report.rs` (empty history, calibration, weekly shares, velocity, the 500-point cap) and `studyfrog-ui/tests/reports_cognitive_ui.rs` (charts and calibration shown; the empty-state message). Checked with a screenshot of a fixture vault (light and dark). The screenshot showed clipped axis labels and a zigzag weekly line; both are fixed: the weekly line is now the cumulative share of passing answers.

Review (2026-11-22): criterion 1 MET: the Cognitive profile tab is on Reports (`/tmp/claude-0/review-c/shots/02-cognitive.png`). Criterion 2 stays unticked, NOT MET: velocity is one line of text, not a curve; the latency chart is a scatter against word count, not a distribution; the weekly chart is a cumulative share of passing answers; no semantic accuracy trend is shown or marked as unavailable (`02-cognitive.png`, `/tmp/claude-0/review-c/shots/03-cognitive-bottom.png`; BUG-0030, BUG-0031). Criterion 3 MET: the calibration shows "Rated Easy 0%" against "Answered in time 100%" with a note that the user may be under-rating recall (`02-cognitive.png`). Status stays review.

## Design Notes
Strictly follow `palette_of(ui)` colors without hardcoded RGB values. Adhere to visual rhythm rules of halves and Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-060, STORY-0176, STORY-0177

## Audit (2026-11-22)
Audited in the running app (display :92, fixture vault). Verdicts:
- Cognitive Profile tab on Reports: MET (`/tmp/claude-0/audit/g2/02-cognitive.png`).
- Charts for velocity, latency and semantic accuracy: NOT MET, so the tick is removed. Velocity is one figure, not a curve; the latency chart is a scatter against word count, not a distribution; no semantic accuracy trend (no semantic scores are recorded). See BUG-0030 and BUG-0031.
- Metacognitive calibration: MET. Rated Easy 33% (21 of 64) and answered in time 36% match the stored rows (`/tmp/claude-0/audit/g2/04-cognitive-bottom.png`, the calibration block in `02-cognitive.png`).
- At 960 by 640 the charts fit; the Reports tab row is cut off (BUG-0032, `/tmp/claude-0/audit/g2/05-cognitive-960.png`).
Open: this story stays in review.

## Open Questions
None.
