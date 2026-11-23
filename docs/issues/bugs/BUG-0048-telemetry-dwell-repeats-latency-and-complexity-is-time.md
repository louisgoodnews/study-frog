---
id: BUG-0048
type: bug
title: "review_telemetry stores dwell_ms equal to latency_ms, and text_complexity holds a reading-time estimate"
status: done
estimate: 2
labels: [m14, store, rehearse]
epic: EPIC-060
parents: ['EPIC-060']
acceptance_criteria:
  - "dwell_ms holds the active reading time of the question and answer, separate from latency_ms (the time before the rating)"
  - "text_complexity holds a word or byte count of the item's text, as STORY-0176 specifies, or the story is changed to say it holds a reading-time estimate"
---

## Context
Found while verifying EPIC-060 (2026-11-22, `/tmp/claude-0/verifyB`). A spaced flip-mode rehearsal of six cards wrote six `review_telemetry` rows (`/tmp/claude-0/verifyB/vault/.studyfrog/index.sqlite`). In every row `dwell_ms` equals `latency_ms` (for example 115 and 115, 117 and 117, 81 and 81). `text_complexity` is 252 in every row for one-word answers, which is a millisecond reading-time estimate, not a word or byte count.

The writer is `crates/studyfrog-service/src/rehearse.rs`, `rate_item_in`: it sets `latency_ms: t.answer_ms` and `dwell_ms: t.answer_ms`, and `text_complexity: t.expected_ms`. STORY-0176 asks for dwell as active reading time, and for latency and text length as separate measures.

## Description
Record dwell as the time the question and answer were on screen, from the run's event log (the item's shown-to-flipped time), and keep latency as the time from the answer being shown to the rating. Store the word or byte count of the item's text in `text_complexity`, and keep the reading-time estimate in its own column or drop it.

## Acceptance Criteria
- [x] `dwell_ms` differs from `latency_ms` when the user reads for a different time from when they rate
- [x] `text_complexity` holds a word or byte count of the item's text, or the story says it holds a reading-time estimate

## Design Notes
- Touchpoints: `crates/studyfrog-service/src/rehearse.rs` (`rate_item_in`), `crates/studyfrog-ui/src/run_update.rs` (`rating_timing`), `crates/studyfrog-store/src/telemetry.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).
- Practice runs (not spaced) write no telemetry rows. That may be intended, since only the schedule-updating preset makes a review; confirm it in EPIC-060.

## Dependencies
- STORY-0176
- EPIC-060

## Verification (2026-11-22)

Review (2026-11-22): NOT FIXED. Criterion 1: all six rows have dwell_ms equal to latency_ms (35/35, 33/33, 34/34, 70/70, 79/79, 57/57; DB `/tmp/claude-0/review-c/vault/.studyfrog/index.sqlite`), and `rehearse.rs` lines 153-154 set both from `t.answer_ms`. Criterion 2: text_complexity is 252 on every row, the `expected_ms` reading-time estimate (`rehearse.rs` line 156). Stays backlog.

Fix (2026-11-23): dwell is the time from the item shown to its rating, paused time excluded (`studyfrog-rehearsal::step::rate` sets `StepOutput.dwell_ms`). Latency is the time from shown to the first response, the flip or the submitted answer (the engine's existing `Latency`). Both are stored apart. Familiarity still uses the reveal-to-rating time (`answer_ms`), unchanged. `text_complexity` is the words of the prompt plus the reference answer, as the store documents it; the reading-time estimate is no longer written to telemetry. The bug text is internally inconsistent about dwell (one line says shown-to-flipped, another the time the question and answer were on screen); the task definition (dwell is the time on the item, latency the time to first response) was followed. Tests: `studyfrog-rehearsal/tests/dwell.rs`, `studyfrog-service/tests/telemetry_fields.rs::dwell_and_latency_are_stored_apart_and_the_complexity_is_the_word_count`. DB check after the fix (same vault): all 10 rows have dwell_ms != latency_ms (row 1: latency 1095, dwell 3141; row 7: 4079 and 6128); text_complexity is the word count (row 1: 5 = "Organelle that makes ATP" 4 + "Mitochondria" 1; row 3: 19; row 6: 18). Before: dwell_ms == latency_ms on all six rows and text_complexity 252 on all (`/tmp/claude-0/review-c/vault/.studyfrog/index.sqlite`).

## Open Questions
- Should practice (non-scheduling) runs write telemetry rows with the effect marked, or none?
