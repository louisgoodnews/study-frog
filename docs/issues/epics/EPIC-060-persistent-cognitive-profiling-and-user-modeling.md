---
id: EPIC-060
type: epic
title: "Persistent Cognitive Profiling, Interaction Telemetry, and User Modeling"
status: done
parents: []
labels: [m14, report, store, srs]
acceptance_criteria:
  - 'Persistent interaction telemetry recording per-item dwell time, response latency, and semantic deviation scores in SQLite'
  - 'Adaptive cognitive profile engine estimating personalized item difficulty and learning velocity'
  - 'Long-term user modeling feeding calibrated priors into FSRS parameter optimization and familiarity scheduling'
  - 'Cognitive profile and memory velocity dashboard integrated into Reports'
---

## Context
Standard spaced repetition algorithms treat memory retention with universal population averages, while classical flashcard apps rely solely on subjective button taps ("Easy" vs "Hard") that get discarded after a run. StudyFrog captures rich implicit behavioral signals—view dwell time, response latency in milliseconds, text complexity, and SLM semantic similarity scores. Persistently storing and modeling these signals enables an adaptive cognitive system that models the individual learner's unique forgetting dynamics and cognitive load.

## Description
Develop a comprehensive Cognitive Profiling and User Modeling subsystem:
1. **Persistent Telemetry Schema**: Record interaction metadata per review event in SQLite, tracking dwell time, response latency, SLM semantic error margins, and text complexity.
2. **Cognitive Profiling Engine**: Compute running estimates of per-item intrinsic difficulty ($D$) and individual learning velocity without modifying raw FSRS invariants.
3. **Adaptive Prior Feed**: Use cognitive telemetry to guide initial item difficulty priors and refine FSRS parameters to the user's measured cognitive half-life.
4. **Learning Insights Visualization**: Provide a dedicated Cognitive Profile view in Reports illustrating retention velocity, accuracy curves across complexity tiers, and metacognitive calibration.

## Acceptance Criteria
- [x] Persistent interaction telemetry recording per-item dwell time, response latency, and semantic deviation scores in SQLite
- [x] Adaptive cognitive profile engine estimating personalized item difficulty and learning velocity
- [x] Long-term user modeling feeding calibrated priors into FSRS parameter optimization and familiarity scheduling
- [x] Cognitive profile and memory velocity dashboard integrated into Reports

## Design Notes
Implement persistence in `studyfrog-store` with SQLite migrations. Implement telemetry analysis as pure free functions in `studyfrog-report` (Invariant P1). Ensure all `.rs` files remain $\le 250$ LOC (Invariant P6).

## Dependencies
None.

## Audit (2026-11-22)
Audited in the running app (display :92). Verdicts per criterion:
- Telemetry recorded per item with dwell, latency and semantic scores: NOT MET. dwell and latency are stored, but `semantic_score` is NULL in `review_telemetry` (BUG-0031). Left unticked.
- Adaptive profile engine (difficulty and velocity): NOT CHECKABLE for the item difficulty (no screen shows it); the velocity is shown (`/tmp/claude-0/audit/g2/02-cognitive.png`). Left unticked.
- Calibrated priors feeding FSRS: NOT CHECKABLE in the app (see STORY-0177). Left unticked.
- Dashboard integrated into Reports: MET (`/tmp/claude-0/audit/g2/02-cognitive.png`). Ticked.
Open: BUG-0030, BUG-0031, BUG-0032. The epic stays in review.

## Verification (2026-11-22)
Checked in the running app (`/tmp/claude-0/verifyB`), a vault with 7 answered reviews (6 spaced flip, 1 spaced typed answer):
- Criterion 1, telemetry in SQLite (dwell, latency, semantic): NOT MET. Dwell and latency are stored, but dwell equals latency on every row and text_complexity is a reading-time estimate (BUG-0048). `semantic_score` is NULL on every row while `answer_score` holds the typed answer's score (BUG-0031). Evidence: `/tmp/claude-0/verifyB/vault/.studyfrog/index.sqlite`, tables `review_telemetry` and `answer_score`.
- Criterion 2, adaptive profile engine (difficulty, velocity): NOT CHECKABLE. Velocity is computed and shown as one number (`/tmp/claude-0/verifyB/r02-cognitive.png`). No screen shows item difficulty. BUG-0030 covers the curve.
- Criterion 3, calibrated priors into FSRS: NOT CHECKABLE in the app. No screen shows the priors or the fitted parameters, and a meaningful check needs many more reviews than this vault has.
- Criterion 4, Cognitive profile in Reports: MET (ticked). The tab opens from Reports (`/tmp/claude-0/verifyB/r02-cognitive.png`, `/tmp/claude-0/verifyB/r03-cognitive-bottom.png`).
The epic stays in review. Open: BUG-0030, BUG-0031, BUG-0048.

Review (2026-11-22): criterion 1 stays unticked, NOT MET: `review_telemetry` rows have dwell_ms equal to latency_ms and semantic_score NULL, including the row for a typed answer whose `answer_score` is 1.0 (BUG-0031, BUG-0048; DB `/tmp/claude-0/review-c/vault/.studyfrog/index.sqlite`). Criterion 2 NOT CHECKABLE: velocity is shown (`/tmp/claude-0/review-c/shots/02-cognitive.png`), but no screen shows per-item difficulty. Criterion 3 NOT CHECKABLE in the app (see STORY-0177). Criterion 4 stays ticked: the Cognitive profile tab opens from Reports (`02-cognitive.png`). Status stays review.

Fix (2026-11-23): criterion 1 is met (ticked). Each timed rating writes one `review_telemetry` row with the dwell (shown to rated), the latency (shown to the first response), the item's words and, for a typed answer, its similarity score from the built-in word matcher (no model installed). Flip-mode rows have no answer to score and stay NULL. BUG-0030, BUG-0031 and BUG-0048 are done. Evidence: DB `/tmp/claude-0/cog/vault/.studyfrog/index.sqlite` (10 rows, dwell differs from latency on all, 5 scores filled); screenshots `/tmp/claude-0/cog/07-cognitive-1280-low.png` and `/tmp/claude-0/cog/10-cognitive-960-semantic.png`. Criterion 2 stays unticked: velocity is now a curve, but no screen shows per-item difficulty (the D_emp index). Criterion 3 stays unticked: the D0 priors have no screen and need two first schedules compared. Status stays review.

## Verification (2026-11-23)
Checked in the running app (`/tmp/claude-0/g4`, the same fixture as STORY-0177: six cards, 60 timed answers over ten days).
- Criterion 1, telemetry in SQLite: MET (ticked earlier). 60 telemetry rows, one per answer, DB `/tmp/claude-0/g4/vault/.studyfrog/index.sqlite`.
- Criterion 2, adaptive profile engine (per-item difficulty and velocity): MET. The Cognitive profile now lists each item's estimated difficulty, from the existing engine (`cognitive_items::item_difficulties`), the slow cards highest: `/tmp/claude-0/g4/shots/final-end.png`. The velocity is shown as before (6.84 passing answers per minute): `/tmp/claude-0/g4/shots/final-top.png`. The item screen (`/tmp/claude-0/g4/shots/item-after.png`) is unchanged; its Difficulty field is the author's mark, not the estimate, so the estimate is on the profile list. Tests: `studyfrog-service/tests/cognitive_items.rs`, `studyfrog-ui/tests/reports_cognitive_ui.rs`.
- Criterion 3, calibrated priors into FSRS: MET (ticked 2026-11-23). The calibrated starting difficulty feeds the live schedule (the stored states of the on and off vaults differ, `/tmp/claude-0/g4/vault/.studyfrog/index.sqlite` and `/tmp/claude-0/g4/vault-off/.studyfrog/index.sqlite`), and the FSRS parameter optimizer now replays each item's first review from the calibrated start when the switch is on (`fsrs_optimize::reviews_to_records_calibrated`, `load_review_records_calibrated`). Checked in the running app, Settings > Rehearsal > Optimize FSRS Parameters: switch on, initial loss 0.0197, final loss 0.0192, RMSE 0.0428 (`/tmp/claude-0/g4/shots/opt-on-metrics.png`); switch off (`calibrated_start = false` in the vault's `studyfrog.toml`), initial loss 0.0201, final loss 0.0195, RMSE 0.0431 (`/tmp/claude-0/g4/shots/opt-off-metrics.png`). Familiarity scheduling takes the stored state, so it gets the calibrated start too (`familiarity::retime_fsrs`). Tests: `studyfrog-service/tests/fsrs_optimize.rs` (`a_calibrated_first_state_differs_only_when_there_are_observations`), `studyfrog-service/tests/calibrated_start.rs`.
  The report replays (backtest, the simulations) still start from the plain prior: they model the schedule, not the user's history, so they are not part of this criterion.
- Criterion 4, Cognitive profile in Reports: MET (ticked earlier).
The epic is done: all four criteria are ticked, and its children STORY-0176, STORY-0177 and STORY-0178 are done. BUG-0030, BUG-0031 and BUG-0048 are done.

## Open Questions
None.
