# Exam mode

A summative exam simulation on top of the rehearsal run ([rehearsal-run.md](rehearsal-run.md)). Formative
modes (`Flip cards`, `Type answers`) let the learner skip, peek and rate by hand; an **exam** does not.
Every answer is written, compared with the reference by the scoring engine ([scoring.md](scoring.md)) and the
run ends with a percentage and a grade. Issues: EPIC-033 (STORY-0125 to STORY-0129).

## 1. Data

| What | Value |
|---|---|
| Run mode | `RunMode::Exam`, persisted name `exam` (next to `flip` and `answer`) in `run.mode` |
| Answers | the existing `AnswerSubmitted(text)` run events (one per item) |
| Item scores | the existing `answer_score` rows (run, item, value 0 to 1, verdict, engine, time) |
| Result | one `exam_result` row per graded run: `run_uid`, `items`, `answered`, `percent` (0 to 100, one decimal), `grade` (letter, e.g. `A-`), `passed` (0/1), `engine`, `graded_at_ms` |

Migration 25 rebuilds the `run` table so its mode check accepts `exam` (all rows are kept) and creates
`exam_result`. Both are history (authoritative, never rebuilt from the vault).

## 2. The plan of an exam

The setup builds the plan as for any run, then the exam rules are applied (`exam_plan`):
- **Kinds:** only cards and questions; notes have no reference to compare with. No kind ticked means both; a
  selection without cards and questions is refused ("an exam needs cards or questions").
- **Effect:** always practice-only. An exam assesses, it does not schedule: there are no ratings, and the
  spacing of reviews is not distorted by a test. The learner's normal runs keep the schedule.
- **Requeue:** off (nothing is rated Again).
- Order, shuffle, limit, stacks, tags, filters and pacing work as for every run.

## 3. Engine rules (strict one-way progression)

In an exam the engine (`studyfrog-rehearsal`, `exam.rs`) accepts only `Start`, `SubmitAnswer(text)`,
`Pause`, `Resume`, `Abort` and `Tick`.

| Rule | How |
|---|---|
| No skipping | `Skip` is refused (`StepError::NotInExam`) |
| No peeking | `Flip` is refused; the answer is never shown during the run |
| No manual rating | `Rate` is refused; there is no rating at all |
| No marks | `Mark` is refused (the exam takes only written answers) |
| No advancing without an answer | `SubmitAnswer` with an empty or blank text is refused (`StepError::EmptyAnswer`); there is no other way forward |
| Answers are locked on submission | the submission emits `AnswerSubmitted` and the next item is shown in the same step; the item never comes back |
| No going back | the cursor only grows; there is no action that moves it back, and the queue never grows |

A refused input leaves the session unchanged, as for every run. Pausing hides the item and stops the clock.

## 4. Scoring an answer

Each submitted answer is scored once, right after submission, with the runtime chosen in the settings
(built-in word matching, or the installed language model), exactly as a typed answer:
- **card:** against `back`;
- **question:** against `reference` and every `accepted` synonym (the best match counts), and the `rubric`
  points (60 % similarity, 40 % share of points covered, see the scoring spec).

The item score is the comparison value in `[0, 1]`. The verdict is shown but does not change the score.

## 5. The exam percentage and the grade

`percent = round(1000 × Σ scores / items) / 10`: the mean item score of **all items of the exam** (each
item once), in percent with one decimal. An item that was not answered (time ran out) or whose answer
could not be scored counts 0. Grades are compared in tenths of a percent, so floating error never moves a
grade.

| Percent | Grade | German | Words |
|---|---|---|---|
| 95 – 100 | `A+` | `1+` | Sehr gut+ (very good) |
| 90 – < 95 | `A` | `1.0` | Sehr gut (very good) |
| 85 – < 90 | `A-` | `1.3` | Sehr gut (very good) |
| 80 – < 85 | `B+` | `1.7` | Gut (good) |
| 75 – < 80 | `B` | `2.0` | Gut (good) |
| 70 – < 75 | `B-` | `2.3` | Gut (good) |
| 65 – < 70 | `C` | `3.0` | Befriedigend (satisfactory) |
| 60 – < 65 | `D` | `4.0` | Ausreichend (sufficient, the minimum to pass) |
| < 60 | `F` | `5.0` | Nicht bestanden (failed) |

**Pass** means at least 60 %. An exam with no items cannot be started (the setup shows 0 items).

## 6. End of an exam

- **Finished** (every item answered): grading starts as soon as every submitted answer has a score or
  failed to get one; the result is stored and shown.
- **Countdown ran out:** the run is finished; unanswered items count 0 and the exam is graded.
- **Stopped** (the Stop button): the run is aborted and **not graded**; the answers given stay in the run's
  history. `Esc` does not stop an exam, so a stray key cannot end it.

## 7. Screens

- **Setup, step 2:** `Exam` next to `Flip cards` and `Type answers`, with a short statement of the rules; the
  schedule switch is replaced by "An exam never changes your review schedule"; the count of items leaves
  notes out.
- **Run:** position, the `Exam` pill, timer and progress; the prompt, one answer field and **Submit and lock
  answer** (disabled while the field is blank). No show-answer, skip, marks or ratings; the hint line states
  the rules.
- **Result:** a banner with the percentage, the grade badge (`A- / 1.3`), Passed or Not passed, answered of
  items and the comparison engine; while scores arrive it shows "Grading…". Below, one card per item: title,
  the written answer, the reference answer and the score with its verdict. **Back to setup** leaves; the
  diary prompt follows with its note pre-filled ("Exam: 82.5 % (B+ / 1.7), passed").
- **Reports > Runs:** the filter offers `Exam`; an exam run shows its percentage, grade and pass or fail.

## 8. Control protocol

`rehearse setup --mode exam`, `rehearse start`, `rehearse type <text>` (the draft), `rehearse submit` (submit
and lock the draft), `rehearse answer <text>` (both at once), `rehearse finish` (leave the result screen),
`rehearse stop`. `flip`, `skip`, `rate` and `mark` answer with an error in an exam, as does a blank answer.
`state` shows `rehearse.run.exam`: items, answered, the locked answers, `grading` and the `result`
(percent, grade, German grade, passed).
