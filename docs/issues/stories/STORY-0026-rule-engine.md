---
id: STORY-0026
type: story
title: "Rule engine"
status: done
epic: EPIC-014
parents: [EPIC-014]
estimate: 5
labels: [m7, achieve]
acceptance_criteria:
  - "Rules are data (scope, metric, threshold) and can be read from achievements.toml"
  - "The evaluator unlocks each rule once, per stack for stack rules"
---

## Context
Rules must be changeable without code.

## Description
Rule engine.

## Acceptance Criteria
- [x] Rules are data (scope, metric, threshold) and can be read from achievements.toml
- [x] The evaluator unlocks each rule once, per stack for stack rules

## Design Notes
Children: TASK-0056, TASK-0057.

## Dependencies
None.

## Open Questions
None.
