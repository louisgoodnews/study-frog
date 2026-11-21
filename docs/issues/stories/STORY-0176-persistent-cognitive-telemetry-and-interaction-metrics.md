---
id: STORY-0176
type: story
title: "Persistent Interaction Telemetry and Cognitive Metrics Schema in SQLite"
status: done
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
- [x] SQLite schema migration adding cognitive interaction telemetry table (review_telemetry)
- [x] Recording of response latency (ms), reading dwell time (ms), SLM semantic similarity score, and text length
- [x] Store query API loading longitudinal interaction telemetry for any item, stack, or time range

## Design Notes
Keep schema lightweight and fast. Write pure query loader functions in `studyfrog-store::telemetry` following Invariant P1 and Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-060

## Implementation (2026-11-21)
Schema step 30 (`review_telemetry`, indexed by item and time), `studyfrog-store::telemetry` (record, load by item list and time range, decode errors are reported). A stack's series is the load for all its item ids. Tests: `studyfrog-store/tests/telemetry.rs`. Recording from the rehearsal screen is not wired yet.

## Open Questions
None.
