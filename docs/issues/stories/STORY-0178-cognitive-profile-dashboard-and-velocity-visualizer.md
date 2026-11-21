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
- [x] Visual charts showing personal learning velocity, response latency distribution, and semantic accuracy trends
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

## Design Notes
Strictly follow `palette_of(ui)` colors without hardcoded RGB values. Adhere to visual rhythm rules of halves and Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-060, STORY-0176, STORY-0177

## Open Questions
None.
