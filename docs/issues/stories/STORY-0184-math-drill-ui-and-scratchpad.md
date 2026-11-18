---
id: STORY-0184
type: story
title: "Interactive Math Drill View with Formula Rendering, Scratchpad, and Step Reveal"
status: backlog
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
- [ ] Clean math rehearsal view with clear formula presentation and given parameter badges
- [ ] Optional intermediate calculation input rows and quick numerical keypad / text input
- [ ] Instant step-by-step solution breakdown revealing correct intermediate steps on demand

## Design Notes
Use `palette_of(ui)` theme tokens. Ensure line count remains $\le 250$ LOC per file (Invariant P6).

## Dependencies
EPIC-062, STORY-0182, STORY-0183

## Open Questions
None.
