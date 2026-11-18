---
id: STORY-0176
type: story
title: "Persistent Interaction Telemetry and Cognitive Metrics Schema in SQLite"
status: backlog
epic: EPIC-060
parents: ['EPIC-060']
estimate: 3
labels: [m14, store, report]
acceptance_criteria:
  - 'SQLite schema migration adding cognitive interaction telemetry table (review_telemetry)'
  - 'Recording of response latency (ms), reading dwell time (ms), SLM semantic similarity score, and text length'
  - 'Store query API loading longitudinal interaction telemetry for any item, stack, or time range'
---

## Context
Granular behavioral metrics gathered during rehearsal runs (such as millisecond response latency and SLM semantic grading deltas) are currently transient. Persisting them in SQLite is necessary for longitudinal user modeling.

## Description
Extend `studyfrog-store` with dedicated persistence for cognitive telemetry:
- Add a new table `review_telemetry` linked to items and review events.
- Capture:
  - `item_uid`: Target item identifier.
  - `reviewed_at`: Unix timestamp of the review.
  - `latency_ms`: Milliseconds elapsed before answer submission.
  - `dwell_ms`: Active reading time spent on the question and answer.
  - `semantic_score`: Float similarity score (0.0 to 1.0) returned by the local SLM grader, if applicable.
  - `text_complexity`: Text byte count or word count proxy.
- Provide efficient indexing and query functions for loading telemetry series.

## Acceptance Criteria
- [ ] SQLite schema migration adding cognitive interaction telemetry table (review_telemetry)
- [ ] Recording of response latency (ms), reading dwell time (ms), SLM semantic similarity score, and text length
- [ ] Store query API loading longitudinal interaction telemetry for any item, stack, or time range

## Design Notes
Keep schema lightweight and fast. Write pure query loader functions in `studyfrog-store::telemetry` following Invariant P1 and Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-060

## Open Questions
None.
