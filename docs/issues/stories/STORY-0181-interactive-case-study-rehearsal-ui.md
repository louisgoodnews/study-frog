---
id: STORY-0181
type: story
title: "Interactive Case-Study Rehearsal UI and Multi-Step Feedback Display"
status: backlog
epic: EPIC-061
parents: ['EPIC-061']
estimate: 3
labels: [m14, ui, rehearse]
acceptance_criteria:
  - 'Responsive case-study rehearsal view rendering scenario vignette with clean typography'
  - 'Dual-input controls for test/procedure selection and multi-line rationale drafting'
  - 'Criterion-by-criterion evaluation breakdown displaying green/red pill badges and diagnostic advice'
---

## Context
Case studies require a specialized UI during rehearsal runs: the student needs to read a multi-paragraph scenario comfortably, input their decision and argument, and see an immediate visual breakdown of which points they nailed and which they missed.

## Description
Implement the interactive case-study view in `crates/studyfrog-ui`:
- Present the vignette in a distinct, well-padded container following visual rhythm rules.
- Render two input areas:
  1. Procedure/Test selector or short text input.
  2. Multi-line text editor for the conceptual justification.
- On submission, display:
  - A summary status badge (e.g. "Solid Decision", "Partial Match", "Alternative Procedure Recommended").
  - An itemized checklist of rubric criteria with checkmarks/crosses.
  - Explanatory feedback explaining why a specific criterion (e.g. social desirability control) was critical.

## Acceptance Criteria
- [ ] Responsive case-study rehearsal view rendering scenario vignette with clean typography
- [ ] Dual-input controls for test/procedure selection and multi-line rationale drafting
- [ ] Criterion-by-criterion evaluation breakdown displaying green/red pill badges and diagnostic advice

## Design Notes
Strictly utilize `palette_of(ui)` theme tokens (success, warning, danger, surface, border). Ensure line counts $\le 250$ LOC per file (Invariant P6).

## Dependencies
EPIC-061, STORY-0179, STORY-0180

## Open Questions
None.
