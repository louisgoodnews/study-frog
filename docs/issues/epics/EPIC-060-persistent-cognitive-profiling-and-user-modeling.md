---
id: EPIC-060
type: epic
title: "Persistent Cognitive Profiling, Interaction Telemetry, and User Modeling"
status: review
parents: []
labels: [m14, report, store, srs]
acceptance_criteria:
  - 'Persistent interaction telemetry recording per-item dwell time, response latency, and semantic deviation scores in SQLite'
  - 'Adaptive cognitive profile engine estimating personalized item difficulty and learning velocity'
  - 'Long-term user modeling feeding calibrated priors into FSRS parameter optimization and familiarity scheduling'
  - 'Cognitive profile and memory velocity dashboard integrated into Reports'
---

## Context
Standard spaced repetition algorithms treat memory retention with universal population averages, while classical flashcard apps rely solely on subjective button taps ("Easy" vs "Hard") that get discarded after a run. StudyFrog captures rich implicit behavioral signals—view dwell time, response latency in milliseconds, text complexity, and SLM semantic similarity scores. Persistently storing and modeling these signals enables an adaptive cognitive system that models the individual learner's unique forgetting dynamics and cognitive load.

## Description
Develop a comprehensive Cognitive Profiling and User Modeling subsystem:
1. **Persistent Telemetry Schema**: Record interaction metadata per review event in SQLite, tracking dwell time, response latency, SLM semantic error margins, and text complexity.
2. **Cognitive Profiling Engine**: Compute running estimates of per-item intrinsic difficulty ($D$) and individual learning velocity without modifying raw FSRS invariants.
3. **Adaptive Prior Feed**: Use cognitive telemetry to guide initial item difficulty priors and refine FSRS parameters to the user's measured cognitive half-life.
4. **Learning Insights Visualization**: Provide a dedicated Cognitive Profile view in Reports illustrating retention velocity, accuracy curves across complexity tiers, and metacognitive calibration.

## Acceptance Criteria
- [ ] Persistent interaction telemetry recording per-item dwell time, response latency, and semantic deviation scores in SQLite
- [ ] Adaptive cognitive profile engine estimating personalized item difficulty and learning velocity
- [ ] Long-term user modeling feeding calibrated priors into FSRS parameter optimization and familiarity scheduling
- [x] Cognitive profile and memory velocity dashboard integrated into Reports

## Design Notes
Implement persistence in `studyfrog-store` with SQLite migrations. Implement telemetry analysis as pure free functions in `studyfrog-report` (Invariant P1). Ensure all `.rs` files remain $\le 250$ LOC (Invariant P6).

## Dependencies
None.

## Audit (2026-11-22)
Audited in the running app (display :92). Verdicts per criterion:
- Telemetry recorded per item with dwell, latency and semantic scores: NOT MET. dwell and latency are stored, but `semantic_score` is NULL in `review_telemetry` (BUG-0031). Left unticked.
- Adaptive profile engine (difficulty and velocity): NOT CHECKABLE for the item difficulty (no screen shows it); the velocity is shown (`/tmp/claude-0/audit/g2/02-cognitive.png`). Left unticked.
- Calibrated priors feeding FSRS: NOT CHECKABLE in the app (see STORY-0177). Left unticked.
- Dashboard integrated into Reports: MET (`/tmp/claude-0/audit/g2/02-cognitive.png`). Ticked.
Open: BUG-0030, BUG-0031, BUG-0032. The epic stays in review.

## Open Questions
None.
