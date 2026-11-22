---
id: STORY-0184
type: story
title: "Interactive Math Drill View with Formula Rendering, Scratchpad, and Step Reveal"
status: review
epic: EPIC-062
parents: ['EPIC-062']
estimate: 3
labels: [m14, ui, rehearse]
acceptance_criteria:
  - 'Clean math rehearsal view with clear formula presentation and given parameter badges'
  - 'Optional intermediate calculation input rows and quick numerical keypad / text input'
  - 'Instant step-by-step solution breakdown revealing correct intermediate steps on demand'
---

## Context
A frictionless UI for quantitative drills needs clear presentation of given parameters, convenient numerical input, and progressive disclosure of hints and solution steps.

## Description
Build the Math Drill screen in `studyfrog-ui`:
- Display the problem scenario (e.g., "Given a test with $s_x = 15$ and $r_{xx} = 0.84$, compute the standard error of measurement $s_e$.").
- Present formatted formula display with highlighted parameter slots.
- Provide input boxes for optional working steps and the final answer.
- Render feedback on submit:
  - Green border and celebratory badge on correct answers.
  - Granular error highlight pointing directly to the mistaken step with constructive remediation advice.
  - "Show full solution" accordion revealing the derivation step-by-step.

## Acceptance Criteria
- [x] Clean math rehearsal view with clear formula presentation and given parameter badges
- [x] Optional intermediate calculation input rows and quick numerical keypad / text input
- [x] Instant step-by-step solution breakdown revealing correct intermediate steps on demand

## Implementation (2026-11-22)
Reached from the rehearse setup's What step: "Math drill" opens a standard-error problem from a seed (`setup::math`, pure state and update; `view::math_view`). The problem shows its given values as badges and the formula. Three optional working rows follow the trace of `studyfrog-rehearsal::formula_trace` (1 - r_xx, its square root, s_e), then the answer. A keypad types into the row last selected. "Check" diagnoses the working rows typed so far with `diagnose_se`, names the first wrong row with its likely mistake, and marks the answer correct within 1 %. "Show full solution" lists each step's value; the values are hidden until then, so the drill does not give the answer away.

The view uses ASCII symbols (`sqrt`, `-`, `*`, `Del`): the bundled font has no glyph for the Unicode signs, and a missing glyph would render as a box.

Open: the engine's trace covers the standard error of measurement only; other formulas of STORY-0182 have no trace yet. The drill is a practice view of its own, not an item kind in a run.

Tests: `studyfrog-ui/tests/math_drill.rs` (opening by seed, correct working, a forgotten root named at its row, the keypad, the solution toggle, the screen text). A screenshot (`--screen math-demo`) shows the mistaken row outlined and the feedback.

## Design Notes
Use `palette_of(ui)` theme tokens. Ensure line count remains $\le 250$ LOC per file (Invariant P6).

## Dependencies
EPIC-062, STORY-0182, STORY-0183

## Open Questions
None.
