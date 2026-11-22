---
id: BUG-0048
type: bug
title: "review_telemetry stores dwell_ms equal to latency_ms, and text_complexity holds a reading-time estimate"
status: backlog
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
- [ ] `dwell_ms` differs from `latency_ms` when the user reads for a different time from when they rate
- [ ] `text_complexity` holds a word or byte count of the item's text, or the story says it holds a reading-time estimate

## Design Notes
- Touchpoints: `crates/studyfrog-service/src/rehearse.rs` (`rate_item_in`), `crates/studyfrog-ui/src/run_update.rs` (`rating_timing`), `crates/studyfrog-store/src/telemetry.rs`.
- Guardrails: P1 (no inherent `impl` blocks) and P2 (≤ 250 lines per `.rs` file).
- Practice runs (not spaced) write no telemetry rows. That may be intended, since only the schedule-updating preset makes a review; confirm it in EPIC-060.

## Dependencies
- STORY-0176
- EPIC-060

## Open Questions
- Should practice (non-scheduling) runs write telemetry rows with the effect marked, or none?
