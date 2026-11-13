---
id: BUG-0006
type: bug
title: "The roadmap is too small"
status: done
epic: EPIC-024
parents: [STORY-0073]
estimate: 2
labels: [m11, ui]
acceptance_criteria:
  - "The roadmap chart takes the width and the height there is, even when the dates are close together"
  - "Rows, bars and text are larger"
---

## Context
The owner sent this after trying 0.3.1 on 2026-11-03.

## Description
With items created today the chart was a few hundred pixels wide in a window of two thousand (screenshot).

## Acceptance Criteria
- [x] The roadmap chart takes the width and the height there is, even when the dates are close together
- [x] Rows, bars and text are larger

## Design Notes
The pixels per day are at least the room divided by the span; the canvas is at least as tall as the room; rows are 38 points, bars 18, text 12 to 13. Test: a two-day bar is wider than 60 points.

## Dependencies
None.

## Open Questions
None.
