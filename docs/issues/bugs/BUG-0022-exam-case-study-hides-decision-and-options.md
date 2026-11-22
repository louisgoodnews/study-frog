---
id: BUG-0022
type: bug
title: "In an exam run, a case study shows only its vignette, without its decision prompt or options"
status: backlog
estimate: 2
labels: [m14, rehearse, ui]
epic: EPIC-061
parents: ['EPIC-061']
acceptance_criteria:
  - "In an exam run, a case study item shows its decision prompt (for example \"Which instrument should the clinic use?\") and its options, as the answer-mode run does"
  - "In an exam run, the student can give the choice and the justification for a case study, or the exam shows that the case is answered as a written answer"
---

## Context

Found while checking EPIC-061 in the running app (2026-11-22). Case studies are in the exam queue (STORY-0199 and EPIC-061 criterion "exam runs"), but the exam view shows only the vignette and a single "Your answer" box:
- the decision prompt and the options are not shown, so the student cannot see what to decide;
- the choice and justification controls of the case view and of the answer-mode run are not shown.

In the answer-mode run the same case shows its options and justification box (`/tmp/claude-0/audit/g3/14-run-item2.png`).

Screenshot: `/tmp/claude-0/audit/g3/21-exam-case.png` (exam, "3 / 5", the case "Choosing a screen for the clinic").

Reproduction: `studyfrog ctl "vault open <folder with a case study>"`, then `ctl "rehearse setup --preset all --stack <stack> --mode exam"` and `ctl "rehearse start"`; step through the items with `ctl "rehearse type x"` and `ctl "rehearse submit"` until the case appears.

## Description

Find where the exam item view (`crates/studyfrog-ui/src/view/run.rs` and the exam view in `exam_ui.rs`) builds the item body, and show the case's decision prompt and options (and, if the exam keeps a typed answer, say so) by reusing the run item's case widgets rather than copying them. Check in the running app at 1280 by 800.

## Acceptance Criteria

None.

## Design Notes

- Guardrails: P4 (nesting < 3) and P6 (files <= 250 lines).
- Do not change the exam grading rules in this bug.

## Dependencies

- EPIC-061
- STORY-0199

## Open Questions

- Whether a case study in an exam is graded from the choice and justification, or from the typed text alone, is not decided here.

## Verification (2026-11-22)

Reproduced. Case "Klinik-Screening" (decision "Which instrument should the clinic use?", options Short screen | Full interview | Self-report scale) in the Diagnostik stack, run as an exam (`rehearse setup --preset all --stack diagnostik --mode exam`). The item shows only the vignette and a "Your answer" box; the decision prompt and the options are missing (`/tmp/claude-0/verifyA/24-exam-run-case.png`). Left in the backlog. The answer-mode comparison was not retaken in this round.

Review (2026-11-22): still NOT FIXED. Current binary, vault `/tmp/claude-0/review-d/vault`, stack `diag`, exam (`rehearse setup --preset all --stack diag --mode exam`, then `rehearse type x` and `rehearse submit` to item 4 of 4). The state reports `prompt` = the vignette only, and the screen shows the vignette with "Your answer" and "Submit and lock answer" only, with no decision prompt and no options (`/tmp/claude-0/review-d/05-exam-case.png`). The same case in answer mode shows the decision, the three options and the justification box (`/tmp/claude-0/review-d/06-answer-case.png`). Status stays backlog.
