---
id: BUG-0031
type: bug
title: "Telemetry rows are saved without a semantic deviation score"
status: backlog
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
- [ ] Every timed review writes a semantic deviation score to review_telemetry, or the telemetry rows state that none was computed for the rating
- [ ] The Cognitive profile uses the stored semantic scores for its accuracy trend once they are recorded

## Design Notes
- None.

## Dependencies
- EPIC-060 (criterion 1), BUG-0030

## Open Questions
- None.
