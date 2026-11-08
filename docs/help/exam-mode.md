# Exam mode

An **exam** is a rehearsal run that works like a written test: every card and question once, answered in writing,
strictly one way, graded at the end. Use it before a real exam to see where you stand. The everyday modes,
**Flip cards** and **Type answers**, are for practice (see [[rehearse]]).

## Starting an exam

1. Open **Rehearse**, choose the stacks (and tags) in step 1 as for any run.
2. In step 2, choose **Exam** next to **Flip cards** and **Type answers**. A countdown makes it a timed exam.
3. Step 3 shows "exam (written answers, graded; cards and questions only)". Press **Start**.

Notes are left out, because they have no answer to grade. An exam **never changes your review schedule**.

## The rules

- You see one item at a time and write your answer in **Your answer**.
- **Submit and lock answer** is the only way forward, and only once you have written something. You cannot skip.
- A submitted answer is **locked**: the next item follows at once and you cannot go back.
- The answer is **not shown** during the exam, and there are no Again / Hard / Good / Easy buttons and no marks.
- **P** pauses (the item is hidden and the clock stops). **Esc** does nothing in an exam, so a stray key cannot end
  it. **Stop exam (not graded)** ends it early; a stopped exam gets no grade.
- When a countdown runs out, the exam ends and every unanswered item counts 0 %.

## How answers are scored

Each answer is compared with the reference the same way as in **Type answers**: a card with its back, a question
with its reference and its `accepted` answers (the best match counts) and its `rubric` key points (60 % similarity,
40 % share of key points covered). The method chosen in [[settings]] does the comparison: the small language model
if it is installed, otherwise word matching. The comparison measures similarity, not truth (see [[faq]]).

Your **exam percentage** is the average score of all items of the exam, with one decimal. Items you did not answer,
and answers that could not be compared, count 0 %.

## The grade

| Percent | Grade | German grade | |
|---|---|---|---|
| 95 to 100 | A+ | 1+ | Sehr gut+ (very good) |
| 90 to under 95 | A | 1.0 | Sehr gut (very good) |
| 85 to under 90 | A- | 1.3 | Sehr gut (very good) |
| 80 to under 85 | B+ | 1.7 | Gut (good) |
| 75 to under 80 | B | 2.0 | Gut (good) |
| 70 to under 75 | B- | 2.3 | Gut (good) |
| 65 to under 70 | C | 3.0 | Befriedigend (satisfactory) |
| 60 to under 65 | D | 4.0 | Ausreichend (sufficient) |
| under 60 | F | 5.0 | Nicht bestanden (failed) |

You **pass** with 60 % or more.

## The result

After the last answer the screen says **Grading…** until every answer is compared, then shows the percentage, the
grade badge (for example **B+ / 1.7**), **Passed** or **Not passed**, and a **Review** with one card per item: your
answer, the reference answer and the score. **Back to setup** (or Space) leaves. The diary prompt below the result
starts with the result as its note, so you can keep it in your [[reports|diary and reports]].

Every exam is listed under **Reports > Runs** with its percentage, grade and whether you passed; choose **Exam** to
see only exams.
