---
id: BUG-0051
type: bug
title: "Math drill says 'Correct. Well worked.' when the final answer is within tolerance but a working step is wrong"
status: backlog
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
