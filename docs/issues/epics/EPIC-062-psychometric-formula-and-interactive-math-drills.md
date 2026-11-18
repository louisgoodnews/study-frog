---
id: EPIC-062
type: epic
title: "Psychometric Formula Engine, Step-by-Step Calculation Drills, and Math Gamification"
status: backlog
parents: []
labels: [m14, rehearse, ui]
acceptance_criteria:
  - 'Domain formula engine supporting standard psychometric equations (standard error, confidence bands, standard scores)'
  - 'Dynamic parameter generator producing infinite synthetic exercise variations from realistic psychological distributions'
  - 'Step-by-step intermediate calculation validator diagnosing exact error locations (formula setup, substitution, arithmetic)'
  - 'Gamified math drill UI with instant calculation feedback, hints, and step reveal'
---

## Context
Psychology examinations in Diagnostics, Statistics, and Methodology feature intensive quantitative components (e.g. standard error of measurement $s_e = s_x \sqrt{1 - r_{xx}}$, confidence intervals $CI = x \pm z \cdot s_e$, standard score conversions $z \leftrightarrow T \leftrightarrow IQ$, and critical difference values). Students frequently memorize final numbers instead of mastering calculation mechanics. An interactive formula drill system with infinite synthetic problem variations and step-by-step diagnosis empowers learners to attain effortless mathematical fluency.

## Description
Build a specialized Psychometric Formula & Math Drill subsystem:
1. **Psychometric Equation Library**: Pure mathematical modeling for standard measurement errors, confidence interval bounds, standard score conversions ($z$, $T$, $Z$, IQ, Stanine), and test reliability corrections (Spearman-Brown).
2. **Infinite Variation Generator**: Procedurally generate randomized realistic parameter values (e.g., sample standard deviations, reliability coefficients $r_{xx} \in [0.70, 0.95]$) so students never run out of practice problems.
3. **Diagnostic Step Validator**: Rather than only checking the final numerical answer, evaluate intermediate stages (e.g. square root computation, multiplier application) to pinpoint precisely where an error occurred.
4. **Interactive Math Practice Interface**: A clean calculation workspace with LaTeX formula rendering, input fields for steps and final solutions, and instant error pinpointing.

## Acceptance Criteria
- [ ] Domain formula engine supporting standard psychometric equations (standard error, confidence bands, standard scores)
- [ ] Dynamic parameter generator producing infinite synthetic exercise variations from realistic psychological distributions
- [ ] Step-by-step intermediate calculation validator diagnosing exact error locations (formula setup, substitution, arithmetic)
- [ ] Gamified math drill UI with instant calculation feedback, hints, and step reveal

## Design Notes
Implement mathematical routines in a dedicated module `studyfrog-rehearsal::psychometrics` or `studyfrog-core` using pure free functions (Invariant P1). Floating point comparisons must use explicit epsilon tolerances (`abs(a - b) < 1e-4`). Maximum 250 LOC per file (Invariant P6).

## Dependencies
None.

## Open Questions
None.
