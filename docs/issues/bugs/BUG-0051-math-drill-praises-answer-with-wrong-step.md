---
id: BUG-0051
type: bug
title: "Math drill says 'Correct. Well worked.' when the final answer is within tolerance but a working step is wrong"
status: review
estimate: 1
labels: [m14, rehearse, ui]
epic: EPIC-062
parents: ['EPIC-062']
acceptance_criteria:
  - "When the final answer is within 1 % but a working row is flagged wrong, the result does not say 'Correct. Well worked.' and does not say 'All steps are correct.'"
  - "When the final answer and every filled working row are correct, the green 'Correct. Well worked.' message stays"
  - "The message for a wrong first working row does not tell the student to 'go back to the step before' when there is no step before"
---

## Context

Found while reviewing EPIC-062 and STORY-0184 in the running app (2026-11-22, `/tmp/claude-0/review-d`). Problem: s_x = 13.3, r_xx = 0.93, correct s_e = 3.519. The working row `1 - r_xx` was typed as 0.3 (wrong), the other rows were right, and the answer was typed as 3.5 (0.5 % off, inside the 1 % tolerance).

Screenshot: `/tmp/claude-0/review-d/20-math-arith.png`. The row `1 - r_xx` is outlined red, but the result box reads "Correct. Well worked." with the line "This step does not match the formula. Go back to the step before and check it." The student is praised and told a step is wrong in the same box.

## Description

The result line is built from the answer check alone, and the working check only adds a line under it. When a working row is wrong, the praise and the step warning must not both appear. Also, the step advice for the first row refers to a step that does not exist.

## Acceptance Criteria

Listed in the front matter.

## Design Notes

- The answer tolerance (1 %) and the row diagnosis (`diagnose_se`) are correct. Only the message text and its choice are wrong.
- Touchpoints: `crates/studyfrog-ui/src/view/math_view.rs` (the result box) and the mistake text in `crates/studyfrog-rehearsal/src/formula_trace.rs` (`mistake_message`).
- Guardrails: P1 (no inherent `impl` blocks), P6 (<= 250 lines per file).

## Dependencies

- EPIC-062
- STORY-0184 (the drill view), STORY-0183 (the step diagnosis)

## Open Questions

- Should a correct final answer with a wrong step show a neutral line ("The answer is right, but one step does not match the formula") or no praise at all? Left to the design of the drill feedback.

## Fix

- Cause: the result box chose its headline from the answer check alone, so a correct final answer with a wrong working row got "Correct. Well worked."; the first-row message also said to go back to the step before.
- Fix: `studyfrog-rehearsal::formula_trace::verdict` returns the headline: praise only when the answer and every submitted row are right; with a wrong row, "The final answer is within tolerance, but a working row is wrong." followed by the name of the first wrong row ("Check the row 1 - r_xx."). The generic step message no longer tells the student to go back to the step before.
- Tests: `crates/studyfrog-rehearsal/tests/formula_verdict.rs` (`a_right_answer_with_a_wrong_first_row_is_not_praised`, `a_right_answer_with_every_row_right_is_praised`, `a_wrong_answer_is_outside_the_tolerance_whatever_the_rows`); `tests/formula.rs` still passes.
- App evidence: s_x 13.3, r_xx 0.7, row `1 - r_xx` typed 0.5, answer 7.28 (within 1 %): the box shows the new headline and the row name, and no "Correct. Well worked.".
- Not checked on screen: the green praise with every row right. The path is the same text for the `WellWorked` verdict, covered by the test.
- Status is `review`, not `done`: its parent EPIC-062 has every child done (STORY-0182, 0183, 0184 and this bug), and the issue rules then require the epic to be done. EPIC-062 has two acceptance criteria still open (the formula engine covers only the standard error, not the confidence bands and standard scores; the drill has no hints). The owner decides whether to close the epic or reopen its criteria.

## Verification (2026-11-23)

The fix holds in the running app (1280x800, copy at `/tmp/claude-0/g2`; s_x 13.9, r_xx 0.76, correct s_e 6.810):
- A wrong first row (`1 - r_xx` typed 0.5) with the answer 6.8 (0.2 % off, inside the 1 %): the headline is "The final answer is within tolerance, but a working row is wrong." and the line is "Check the row 1 - r_xx."; "Correct. Well worked." does not appear (`/tmp/claude-0/g2/shot-44-bug-check.png`).
- Every row right (0.24, 0.49, 6.81) and the answer 6.8: "Correct. Well worked." and "All steps are correct." (`/tmp/claude-0/g2/shot-47-praise.png`). The praise path with every row right, which the 2026-11-22 note left unchecked, is now seen on screen.
- A wrong second row (`sqrt(1 - r_xx)` typed 0.46): the headline is the same warning, the line names "Check the row sqrt(1 - r_xx).", and there is no praise (`/tmp/claude-0/g2/shot-46-bug-praise.png`).
- The first-row message no longer tells the student to go back: it reads "This step does not match the formula. Check it against the formula and try again." (`mistake_message`, `Mistake::Unexplained`).

Status stays review, as the owner asked; the epic cannot be done until this is done.
