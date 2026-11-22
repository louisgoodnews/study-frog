---
id: STORY-0180
type: story
title: "SLM Multi-Criteria Semantic Evaluator for Decision Choices and Conceptual Rationales"
status: done
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
- [x] Dual-pass prompt template evaluating both categorical selection and free-text justification against rubric
- [x] Structured JSON extraction parsing decision accuracy, rubric points met, and missing caveats
- [x] Deterministic fallback scoring for offline operations without active SLM backend

## Design Notes
Keep all prompt construction and response parsing in pure functions without inherent `impl` blocks (Invariant P1). Handle malformed SLM JSON output gracefully.

## Dependencies
EPIC-061, STORY-0179

## Implementation (2026-11-21)
`studyfrog-agent::rubric_eval`: parsing the model's JSON (matched to the rubric by text), a recomputed overall score (40 % choice, 60 % rubric), and the deterministic keyword fallback when no model runs. Tests: `tests/rubric_eval.rs`. The dual-stage prompt template and the caveats field are done in STORY-0200, and the case view calls the local model.

Audit (2026-11-22): all three criteria MET, checked in the running app with a local stub endpoint standing in for a model (no real model is installed, so model quality is not checked here). (1) The two-stage prompt (`rubric_eval::case_system`: choice, then each criterion, then one JSON object) was sent to the local provider (`/tmp/claude-0/audit/g3/08-model-good.png`). (2) The reply's JSON gave choice validity, criteria met and caveats (`/tmp/claude-0/audit/g3/08-model-good.png`, `/tmp/claude-0/audit/g3/10-model-caveat.png`); a reply without JSON kept the keyword check and said why (`/tmp/claude-0/audit/g3/11-model-garbage.png`). (3) With no model, the keyword check gave the right choice and two of three points 80 % (`/tmp/claude-0/audit/g3/03-case-checked.png`), the wrong choice 40 % (`/tmp/claude-0/audit/g3/06-wrong-checked.png`), and a shorter justification 60 % (`/tmp/claude-0/audit/g3/11-model-garbage.png`); each says "without a model".

## Open Questions
None.
