---
id: STORY-0181
type: story
title: "Interactive Case-Study Rehearsal UI and Multi-Step Feedback Display"
status: done
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
- [x] Responsive case-study rehearsal view rendering scenario vignette with clean typography
- [x] Dual-input controls for test/procedure selection and multi-line rationale drafting
- [x] Criterion-by-criterion evaluation breakdown displaying green/red pill badges and diagnostic advice

## Implementation (2026-11-21)
Reached from the rehearse setup's What step: "Case study" opens the valid cases of the stack that is open (`setup::case`, pure state and update; `view::case_view`). Each case shows its vignette in a framed block, its decision with the options as checkboxes, and a multi-line justification. "Check my answer" scores the choice (`choice_is_correct`) and the justification against the rubric (`rubric_eval::fallback_evaluate`). The result shows a status pill (Solid decision, Partial match, Alternative procedure recommended, Needs more justification), the score, a choice pill, and a Met or Missing pill per criterion with its feedback.

Open: the view is a practice screen of its own, not a step inside a rehearsal run; cases in a run's queue are STORY-0199. A local model checks the justification when one is selected (STORY-0200).

Tests: `studyfrog-ui/tests/case_study_ui.rs` (opening, scoring, next, view text). A screenshot (`--screen case-demo`) shows the layout and the pills.

Audit (2026-11-22): all three criteria MET in the running app. (1) The vignette sits in a framed block at 1280x800 and 960x640 (`/tmp/claude-0/audit/g3/02-case-list.png`, `/tmp/claude-0/audit/g3/07-case-960.png`). (2) Dual inputs: option checkboxes and a multi-line justification box (`/tmp/claude-0/audit/g3/02-case-list.png`). (3) After "Check my answer": status pill, score, choice pill, and a green Met or red Missing pill per criterion with its feedback and the "Consider:" advice (`/tmp/claude-0/audit/g3/03-case-checked.png`, `/tmp/claude-0/audit/g3/04-case-scrolled.png`, `/tmp/claude-0/audit/g3/06-wrong-checked.png` for "Alternative procedure recommended"). Defect filed: the Case study button ignores the ticked stack (BUG-0023, `/tmp/claude-0/audit/g3/08a-setup.png`).

## Design Notes
Strictly utilize `palette_of(ui)` theme tokens (success, warning, danger, surface, border). Ensure line counts $\le 250$ LOC per file (Invariant P6).

## Dependencies
EPIC-061, STORY-0179, STORY-0180

## Open Questions
None.
