---
id: TASK-0062
type: task
title: "Report screens and benchmark"
status: done
epic: EPIC-015
parents: [STORY-0029]
estimate: 3
labels: [m7, ui, bench]
acceptance_criteria:
  - "A Reports screen with retention, time to answer and reviews ahead; every bar also shows its numbers as text"
  - "A benchmark on about 100,000 reviews, each report under 50 ms"
---

## Context


## Description
Report screens and benchmark.

## Acceptance Criteria
- [x] A Reports screen with retention, time to answer and reviews ahead; every bar also shows its numbers as text
- [x] A benchmark on about 100,000 reviews, each report under 50 ms

## Design Notes
`ui::view::reports`, `service::reports`, `report/benches/reports.rs`: 0.38 ms, 1.3 ms and 6.6 ms for 103,596 reviews.

## Dependencies
TASK-0061.

## Open Questions
None.
