---
id: STORY-0160
type: story
title: "Local FSRS-6 Parameter Optimization Engine"
status: backlog
epic: EPIC-054
parents: ['EPIC-054']
estimate: 5
labels: [m13, srs]
acceptance_criteria:
  - 'Pure Rust Adam gradient descent optimizes 21 FSRS-6 weights on local SQLite reviews'
  - 'L2 regularization penalizes drift from default priors to prevent overfitting'
  - "Settings screen offers 1-click 'Optimize Parameters' button with before/after log-loss metrics"
---

## Context
Personalizing the FSRS-6 spaced repetition scheduler to individual learners.

## Description
Implement a local, dependency-free Adam optimizer in Rust that trains personal FSRS-6 weights directly from historical review logs in SQLite.

## Acceptance Criteria
- [ ] Pure Rust Adam gradient descent optimizes 21 FSRS-6 weights on local SQLite reviews
- [ ] L2 regularization penalizes drift from default priors to prevent overfitting
- [ ] Settings screen offers 1-click 'Optimize Parameters' button with before/after log-loss metrics

## Design Notes
See docs/v0.9.0-roadmap.md.

## Dependencies
EPIC-054

## Open Questions
None.
