---
id: STORY-0178
type: story
title: "Cognitive Profile Dashboard and Learning Velocity Visualization in Reports"
status: backlog
epic: EPIC-060
parents: ['EPIC-060']
estimate: 3
labels: [m14, ui, report]
acceptance_criteria:
  - 'Cognitive Profile tab added to the Reports screen'
  - 'Visual charts showing personal learning velocity, response latency distribution, and semantic accuracy trends'
  - 'Metacognitive calibration widget comparing subjective user ratings against objective telemetry metrics'
---

## Context
Learners benefit from transparent insight into how their memory actually operates: whether they over-rate their mastery, how fast they recall different subjects, and how their cognitive velocity evolves across semesters.

## Description
Build the Cognitive Profile visualization in `studyfrog-ui`:
- Add a new tab/subview under Reports displaying the learner's cognitive profile summary.
- Render charts using `egui` painter primitives and palette tokens (`palette_of(ui)`):
  - Learning velocity curve (stability growth per review minute).
  - Latency vs text complexity scatter/distribution.
  - Semantic accuracy progression over time.
- Display a Metacognitive Calibration score highlighting discrepancies between subjective ratings ("Easy") and objective latencies.

## Acceptance Criteria
- [ ] Cognitive Profile tab added to the Reports screen
- [ ] Visual charts showing personal learning velocity, response latency distribution, and semantic accuracy trends
- [ ] Metacognitive calibration widget comparing subjective user ratings against objective telemetry metrics

## Design Notes
Strictly follow `palette_of(ui)` colors without hardcoded RGB values. Adhere to visual rhythm rules of halves and Invariant P6 ($\le 250$ LOC).

## Dependencies
EPIC-060, STORY-0176, STORY-0177

## Open Questions
None.
