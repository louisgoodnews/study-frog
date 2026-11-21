---
id: STORY-0183
type: story
title: "Step-by-Step Intermediate Calculation Validator and Error Diagnostics"
status: done
epic: EPIC-062
parents: ['EPIC-062']
estimate: 5
labels: [m14, rehearse]
acceptance_criteria:
  - 'Step-by-step calculation trace generator breaking formulas into discrete sub-operations'
  - 'Diagnostic evaluator identifying the exact point of deviation in user-submitted steps'
  - 'Specific feedback messages explaining common student errors (e.g. subtracting variance instead of standard deviation)'
---

## Context
When a student miscalculates an exam problem, an app that only says "Wrong! The answer is 4.12" provides minimal educational value. Identifying that the error happened in Step 2 (e.g. failing to take the square root of $1 - r_{xx}$) mimics the guidance of an expert tutor.

## Description
Build the diagnostic step validator:
- Define `FormulaStepTrace` containing the sequence of operations for an exercise:
  - Step 1: Subtraction under root ($1 - r_{xx}$).
  - Step 2: Radical evaluation ($\sqrt{1 - r_{xx}}$).
  - Step 3: Multiplication by standard deviation ($s_x \cdot \dots$).
- When the student enters their intermediate working and final answer, evaluate each stage.
- If an intermediate value diverges, diagnose the typical mistake:
  - Wrong $z$-value for chosen confidence level ($\alpha = 0.05 \implies z = 1.96$ vs $z = 1.64$ one-tailed).
  - Inverted fraction or forgotten root.
  - Rounding error vs conceptual mistake.

## Acceptance Criteria
- [x] Step-by-step calculation trace generator breaking formulas into discrete sub-operations
- [x] Diagnostic evaluator identifying the exact point of deviation in user-submitted steps
- [x] Specific feedback messages explaining common student errors (e.g. subtracting variance instead of standard deviation)

## Design Notes
Keep verification pure and deterministic. Return structured diagnostic error enums adhering to Invariant P1.

## Dependencies
EPIC-062, STORY-0182

## Implementation (2026-11-21)
`studyfrog-rehearsal::formula_trace`: the three-step trace of the standard error, `diagnose_se` (first deviating step, forgotten root, variance used for the standard deviation, inverted subtraction, rounding, unexplained), and the feedback sentences. Tests: `tests/formula.rs`.

## Open Questions
None.
