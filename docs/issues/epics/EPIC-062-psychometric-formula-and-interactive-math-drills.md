---
id: EPIC-062
type: epic
title: "Psychometric Formula Engine, Step-by-Step Calculation Drills, and Math Gamification"
status: review
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
- [x] Dynamic parameter generator producing infinite synthetic exercise variations from realistic psychological distributions
- [x] Step-by-step intermediate calculation validator diagnosing exact error locations (formula setup, substitution, arithmetic)
- [ ] Gamified math drill UI with instant calculation feedback, hints, and step reveal

## Design Notes
Implement mathematical routines in a dedicated module `studyfrog-rehearsal::psychometrics` or `studyfrog-core` using pure free functions (Invariant P1). Floating point comparisons must use explicit epsilon tolerances (`abs(a - b) < 1e-4`). Maximum 250 LOC per file (Invariant P6).

## Dependencies
None.

## Open Questions
None.

## Verification (2026-11-22)

Review (2026-11-22), running app (`/tmp/claude-0/review-d`, current head binary; the drill is reached from rehearse setup, "Math drill"):
- Criterion 1 (formula engine: SE, confidence bands, standard scores): NOT CHECKABLE for the criterion as written. The app's drill uses only the standard error (`s_e = s_x * sqrt(1 - r_xx)`). Its values match the hand calculation: s_x 13.3, r_xx 0.93 gives 1 - r_xx = 0.070, sqrt = 0.265, s_e = 3.519 (`/tmp/claude-0/review-d/19-solution.png`; 13.3 * sqrt(0.07) = 3.5188). The confidence-band and standard-score functions have no path in the UI, so the app cannot show them. Left unticked.
- Criterion 2 (generator): MET. Four New problem clicks gave different problems within the story's ranges: s_x 10 with r_xx 0.95, s_x 13.3 with r_xx 0.93 (`/tmp/claude-0/review-d/18-newproblem-1.png`, `18-newproblem-3.png`); `formula_gen.rs` draws r_xx in [0.65, 0.98], s_x in [10, 15], means 40 to 60, from a u64 seed.
- Criterion 3 (step validator): MET. Forgotten root (sqrt row left at 0.23 for s_x 11.4, r_xx 0.77): the sqrt row is outlined red with "You left out the square root" (`/tmp/claude-0/review-d/17-math-forgot-root.png`). Arithmetic error (1 - r_xx typed 0.3): the first row is outlined red (`/tmp/claude-0/review-d/20-math-arith.png`). Defect in the same box: BUG-0051 (the result reads "Correct. Well worked." because the answer is within 1 %).
- Criterion 4 (gamified UI with instant feedback, hints, step reveal): NOT MET. Instant feedback MET (correct working gives the green "Correct. Well worked." box, `/tmp/claude-0/review-d/16-math-correct.png`). Step reveal MET ("Show full solution" lists 1 - r_xx = 0.070, sqrt(1 - r_xx) = 0.265, s_e 3.519, `/tmp/claude-0/review-d/19-solution.png`). Hints: NOT MET. The drill has no hint control or hint text; its controls are Check, Show full solution, New problem and Close drill (`/tmp/claude-0/review-d/14-math.png`). Left unticked.

Status stays review: criteria 1 and 4 are open. Children: STORY-0182 done, STORY-0183 done, STORY-0184 done.
