---
id: BUG-0031
type: bug
title: "Telemetry rows are saved without a semantic deviation score"
status: done
estimate: 2
labels: [m14, store, report]
epic: EPIC-060
parents: ['EPIC-060']
acceptance_criteria:
  - "Every timed review writes a semantic deviation score to review_telemetry, or the telemetry rows state that none was computed for the rating"
  - "The Cognitive profile uses the stored semantic scores for its accuracy trend once they are recorded"
---

## Context
Found while auditing EPIC-060 (2026-11-22, display :92). A flip-mode rating in the running app wrote a `review_telemetry` row with dwell_ms and latency_ms set and `semantic_score` NULL (vault DB `.studyfrog/index.sqlite`, table `review_telemetry`). EPIC-060's first criterion asks for semantic deviation scores in SQLite.

## Description
Record the semantic deviation for each review that has one (the SLM answer path), and leave the column NULL only for reviews without an answer to score. Add a test that a typed answer writes a score.

## Acceptance Criteria
- [x] Every timed review writes a semantic deviation score to review_telemetry, or the telemetry rows state that none was computed for the rating
- [x] The Cognitive profile uses the stored semantic scores for its accuracy trend once they are recorded

## Design Notes
- None.

## Dependencies
- EPIC-060 (criterion 1), BUG-0030

## Verification (2026-11-22)
Reproduced in the running app (`/tmp/claude-0/verifyB`). A spaced rehearsal (`--preset spaced`, which is the only preset that updates the schedule and so writes telemetry) with typed answers:
- `answer_score` holds the typed answer's score: value 1.0, verdict `correct`, engine "word matching (built in)" (row 1 in `/tmp/claude-0/verifyB/vault/.studyfrog/index.sqlite`).
- The matching `review_telemetry` row (id 7) has `latency_ms` 199, `dwell_ms` 199, `semantic_score` NULL and no marker that no score was computed. The same NULL holds for all six flip-mode rows (ids 1 to 6).
- The only writer of `review_telemetry` sets `semantic_score: None` (`crates/studyfrog-service/src/rehearse.rs`, `rate_item_in`), so the score is never copied from `answer_score`.
No model is needed for this: the built-in word matcher produced the score. Status stays backlog.

Review (2026-11-22): NOT FIXED. Criterion 1: the typed answer for item 01M82N31YN67WE67F8SMTF1TV9 (Alpha 1) has an `answer_score` of 1.0 (correct), but its `review_telemetry` row (id 4) has semantic_score NULL and no marker (DB `/tmp/claude-0/review-c/vault/.studyfrog/index.sqlite`). `crates/studyfrog-service/src/rehearse.rs` line 155 still writes `semantic_score: None`. Criterion 2: no accuracy trend is shown (`/tmp/claude-0/review-c/shots/03-cognitive-bottom.png`). Stays backlog.

Fix (2026-11-23): the rating's timing carries the typed answer's similarity score. `studyfrog-ui::run_update::timing::rating_timing` reads `run.scored` before a rating clears it, and `studyfrog-service::rehearse::rate_item_in` writes it as `semantic_score` (clamped to [0, 1]). Source: the built-in word matcher (`engine: word matching (built in)`); no model is installed here. A flip-mode answer has no answer to score, so its row stays NULL, which is the case the bug's description leaves NULL. Tests: `studyfrog-service/tests/telemetry_fields.rs::a_judged_answer_records_its_semantic_score`. DB check (`/tmp/claude-0/cog/vault/.studyfrog/index.sqlite`, table `review_telemetry`, after the fix): rows 6 to 10 (typed) have semantic_score 1.0, 1.0, 1.0, 0.4603, 0.8292, equal to the five `answer_score` rows of the same items (values 1.0, 1.0, 1.0, 0.4603, 0.8292); rows 1 to 5 (flip) are NULL. Before: all rows NULL (`/tmp/claude-0/review-c/vault/.studyfrog/index.sqlite`).

## Open Questions
- None.
