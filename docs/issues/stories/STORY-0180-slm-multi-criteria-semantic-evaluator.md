---
id: STORY-0180
type: story
title: "SLM Multi-Criteria Semantic Evaluator for Decision Choices and Conceptual Rationales"
status: backlog
epic: EPIC-061
parents: ['EPIC-061']
estimate: 5
labels: [m14, agent, slm]
acceptance_criteria:
  - 'Dual-pass prompt template evaluating both categorical selection and free-text justification against rubric'
  - 'Structured JSON extraction parsing decision accuracy, rubric points met, and missing caveats'
  - 'Deterministic fallback scoring for offline operations without active SLM backend'
---

## Context
When a student answers a case study, checking only the final choice or performing basic string matching misses whether they actually understood the underlying psychological reasoning. A local SLM can evaluate whether the conceptual justification satisfies each rubric requirement.

## Description
Develop the semantic evaluation pipeline in `studyfrog-agent`:
- Formulate a robust system prompt for the local SLM that receives:
  - The case vignette.
  - The student's chosen test/decision.
  - The student's free-form explanation.
  - The target rubric criteria.
- Instruct the SLM to output structured JSON:
  - `choice_valid`: Boolean or categorical match.
  - `criteria_results`: List of `(criterion_text, met: bool, feedback: String)`.
  - `overall_score`: Aggregate percentage (0 to 100%).
- Provide a pure string-similarity fallback when the SLM is offline.

## Acceptance Criteria
- [ ] Dual-pass prompt template evaluating both categorical selection and free-text justification against rubric
- [ ] Structured JSON extraction parsing decision accuracy, rubric points met, and missing caveats
- [ ] Deterministic fallback scoring for offline operations without active SLM backend

## Design Notes
Keep all prompt construction and response parsing in pure functions without inherent `impl` blocks (Invariant P1). Handle malformed SLM JSON output gracefully.

## Dependencies
EPIC-061, STORY-0179

## Open Questions
None.
