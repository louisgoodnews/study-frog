---
id: STORY-0177
type: story
title: "Adaptive Cognitive Difficulty and Learning Velocity Estimation Engine"
status: done
epic: EPIC-060
parents: ['EPIC-060']
estimate: 5
labels: [m14, srs, report]
acceptance_criteria:
  - 'Pure mathematical engine computing empirical item difficulty from latency, dwell time, and semantic accuracy'
  - 'User-level learning velocity estimator quantifying personal memory consolidation rate over time'
  - 'Automated calibration of initial item difficulty priors (D0) based on cognitive telemetry'
---

## Context
Fixed difficulty baselines fail to reflect that different learners process conceptual density, technical terminology, and formulaic tasks at vastly different speeds. Empirical telemetry allows computing personalized difficulty and learning velocity.

## Description
Develop a mathematical engine in `studyfrog-report` (or `studyfrog-srs`):
- Compute an empirical difficulty index $D_{\text{emp}} \in [1, 10]$ derived from normalized response latency, relative dwell time compared to text length, and SLM semantic correctness.
- Formulate a learning velocity metric: the rate at which stability $S$ grows per unit of study time invested.
- Feed calibrated item difficulty into initial memory state estimation (`initial_state`), preventing over-scheduling of deceptively long or under-recalled cards.

## Acceptance Criteria
- [x] Pure mathematical engine computing empirical item difficulty from latency, dwell time, and semantic accuracy
- [x] User-level learning velocity estimator quantifying personal memory consolidation rate over time
- [x] Automated calibration of initial item difficulty priors (D0) based on cognitive telemetry

## Design Notes
Implement strictly as free functions adhering to Invariant P1 (zero inherent `impl` blocks). Maintain deterministic numerical stability with thorough unit and property tests.

## Dependencies
EPIC-060, STORY-0176

## Implementation (2026-11-21)
`studyfrog-report::difficulty`: D_emp on [1, 10], the velocity slope, and `calibrated_prior` (the prior counts as five observations). Tests: `tests/difficulty.rs`. The Cognitive profile tab (STORY-0178) shows a consolidation chart: the mean days between passing reviews, by week (`studyfrog-service::cognitive_report`). The history keeps no stability, so the gap stands in for it. Test: `studyfrog-service/tests/cognitive_report.rs`; the tab's section is checked by `studyfrog-ui/tests/reports_cognitive_ui.rs` (the screenshot does not reach it below the fold).

The calibrated starting difficulty is on by default, and the Scheduling section of Settings has a checkbox for it (it writes `calibrated_start` under `[srs]` in `studyfrog.toml`, keeping the rest of the file; `studyfrog-service::srs_setting`). A first FSRS review blends the scheduler's prior with the difficulties of the user's earlier first answers (`studyfrog-service::schedule::calibrated_initial`, the last 500 first reviews that have timing). With no timed history the prior is used unchanged, so new users get the same schedule as before. Rerating (`rerate.rs`) still replays with the plain prior;

Tests: `studyfrog-service/tests/calibrated_start.rs` (prior unchanged without data, direction of the blend, one timing per item), `studyfrog-service/tests/calibrated_switch.rs` (the switch is written, kept in the file and read back), `studyfrog-config/tests/srs_switch.rs`, and `studyfrog-ui/tests/srs_switch_ui.rs`.

Review (2026-11-22): criterion 1 (D_emp engine) stays ticked, NOT CHECKABLE in the app: no screen shows the index and the unit tests were not run here. Criterion 2 (velocity) MET: the Cognitive profile shows 1031.79 passing answers per minute (`/tmp/claude-0/review-c/shots/02-cognitive.png`). Criterion 3 (calibrated D0 priors) NOT CHECKABLE: confirming its effect needs two first schedules compared with the setting on and off, not done here. Status stays review.

## Audit (2026-11-22)
Audited in the running app (display :92, fixture vault with 64 timed reviews over 10 days, values checked against the rows). Verdicts:
- Empirical difficulty engine (D_emp): NOT CHECKABLE in the app. No screen shows the index; the engine is pure and its tests were not run in this audit. Left as ticked by the owner.
- Learning velocity and consolidation over time: MET. The Cognitive profile shows the velocity (61.15 passing answers per minute) and days between passing reviews by week (`/tmp/claude-0/audit/g2/02-cognitive.png`, `/tmp/claude-0/audit/g2/04-cognitive-bottom.png`). The velocity is one figure, not a curve (BUG-0030).
- Calibrated D0 priors: NOT CHECKABLE. The Settings switch exists (per the notes), but its effect on the first schedule needs a comparison of two schedules, not done here. Left unticked.
Open: this story stays in review.

## Verification (2026-11-23)
Checked in the running app (`/tmp/claude-0/g4`, a vault with one stack of six cards rated on ten backdated days, 60 timed answers; the same history built twice, with the calibrated start on and off).
- Criterion 1, D_emp: MET. The Cognitive profile lists each item with its estimated difficulty, the slow cards highest (5.8 and 5.5 against about 2.8 for the quick ones): `/tmp/claude-0/g4/shots/final-end.png`. Tests: `studyfrog-service/tests/cognitive_items.rs` (the figure is the engine's mean over the answers), `studyfrog-report` difficulty tests.
- Criterion 2, learning velocity: MET (ticked earlier). The figure is 6.84 passing answers per minute, with the curve: `/tmp/claude-0/g4/shots/final-top.png`.
- Criterion 3, calibrated D0 priors: MET. The first review of each item gets its starting difficulty blended with the earlier first answers. Evidence in the stored schedule: with the start on (`/tmp/claude-0/g4/vault/.studyfrog/index.sqlite`) and off (`/tmp/claude-0/g4/vault-off/.studyfrog/index.sqlite`), the first item (no earlier answers) has the same difficulty and stability (2.06, 40.32 days) in both, and the other five differ (for example the Osmosis card, difficulty 8.34 against 8.51, stability 20.7 against 19.2 days). The first review's stability depends on the rating only, so the difference shows from the second review on.
- Schedule comparison (the criterion as worded, adaptive against fixed): MET by a new section on the Cognitive profile, "Adaptive schedule against the fixed one (second review)": 6 items, interval 6.8 days under both (0 of 6 differ at whole days), stability 6.98 days fixed against 6.81 days calibrated (the harder items get the lower stability). Screenshot: `/tmp/claude-0/g4/shots/final-end.png`. Tests: `studyfrog-service/tests/schedule_compare.rs` (the calibrated stability is lower after slow first answers; no timing gives the same schedule), `studyfrog-service/tests/calibrated_start.rs`, `studyfrog-ui/tests/reports_cognitive_ui.rs`.

The FSRS parameter optimizer also takes the calibrated start when the switch is on (`fsrs_optimize::load_review_records_calibrated`), checked in the running app: initial loss 0.0197 against 0.0201 with the switch off, RMSE 0.0428 against 0.0431 (`/tmp/claude-0/g4/shots/opt-on-metrics.png`, `opt-off-metrics.png`).

## Open Questions
None.
