# Rehearsing

A rehearsal is a **run**: StudyFrog shows you items one by one, you recall the answer and rate how well it went.
Open **Rehearse** in the sidebar, press **Start today's rehearsal** on [[home|Home]], or **Start rehearsal** on a
[[stack-page|stack page]].

## Setting up a run

The setup has three steps. **Step 1 of 3** and so on shows where you are; **Back** goes a step back.

### 1. What

- **Choose the stacks**: tick stacks, or leave all unticked for everything. A ticked stack includes the stacks
  inside it.
- **Kinds of item**: Cards, Notes, Questions. Nothing ticked means all kinds.
- **Tags (any of, comma separated)**.

The setup counts the matching items as you choose.

### 2. How

- **Presets**: **Spaced repetition**, **Everything, in order**, **Everything, shuffled**, **Hardest first** and
  **Not seen for longest**. Start from one, then adjust.
- **Time**: **No clock**, **Countdown** (5, 10, 15 or 25 minutes, or any number from 1 to 180) or **Count up**.
- **Flip cards** (you think of the answer, then show it) or **Type answers** (you type your answer and StudyFrog
  compares it with the reference).
- **Order**: Due first, In order, Hardest first, Easiest first, Not seen for longest, Most likely forgotten, or
  Least familiar first.
- **Shuffle the order of the chosen items**: works with every order. It changes the sequence, never which items
  are chosen or whether the schedule changes.
- **Maximum number of items**.
- **More options**: **Update my review schedule with these ratings**, **Show items I rate Again once more in this
  run**, which items to include (Any, New only, Already learned, Due now, Not due yet), **Only items within a
  difficulty range**, and **Only items I am less familiar with than a level**.

### 3. Review

Check your choices and press **Start rehearsal**.

## Spaced runs and practice runs

- **Due first** (spaced repetition) shows the items that are due, most overdue first, then new items, up to your
  daily limits. Higher priority goes first. Items and stacks that are Complete or On hold are skipped. Your
  ratings update the schedule.
- Every other order goes through **all** matching items. These runs are **practice only** by default: your ratings
  and timings are recorded, but the schedule does not change. Tick **Update my review schedule with these
  ratings** if you want them to count.

## During the run

The card shows the prompt, a progress bar and, with a clock, the time left or used.

- **Show answer (Space)**, then rate: **Again (1)**, **Hard (2)**, **Good (3)**, **Easy (4)**.
- **Skip (S)** passes an item. **Pause (P)** hides the question and stops the clock; **Resume (P)** goes on.
- **Stop (Esc)** ends the run early. What you rated so far is saved.
- **This item is:** **Easy (E)**, **Medium (M)** or **Difficult (D)**. A mark records how the item felt. It
  never changes the schedule by itself, but it feeds suggestions, reports and [[familiarity]].
- An item made from a PDF has a **Source** link to its page.

With a countdown, "One minute left" and "Ten seconds left" are shown in text and announced to screen readers. At
zero the run ends by itself. The item on screen then counts as not answered: it gets no rating and its schedule
does not change.

All keys are listed in [[keyboard|Keyboard shortcuts]].

## Typed answers

Type your answer and press **Submit answer**. StudyFrog shows the reference answer, how closely yours matched as a
percentage, a verdict (**Correct**, **Partly right** or **Not quite**), a suggested rating, and what compared the
texts (**Compared by**). You still choose the rating.

The comparison measures **similarity, not truth**. An answer that differs from the reference by one decisive word
("raises" instead of "lowers", 1889 instead of 1789) usually scores as correct. Treat the match as a hint. Word
matching is built in; a small language model gives better results if installed. See [[settings]] and the [[faq]].

## After the run

The review shows tiles and bars for your ratings, the mode, the pacing, the planned time of a countdown, the time
used and your answers per minute. Below, every item has its own card with your previous rating, your mark, your
rating and the typed-answer match.

StudyFrog may **suggest** a different rating:

- you read the answer much faster than your reading speed predicts: a Good may have been Easy;
- a new item went by very fast, or you read very slowly: it may be harder than you rated it;
- you marked it difficult but rated it Good or Easy, or marked it easy and rated it Good.

In a run that updates the schedule, press the suggestion to change the rating; the schedule of that item is rebuilt
as if you had pressed it then. Practice runs show suggestions but cannot apply them. Measure your own reading speed
in [[settings]] to make suggestions fit you.

**Back to setup** starts the next run.

## If StudyFrog closes during a run

What you did is saved after every item. A run left open by a crash is saved as stopped the next time you open the
vault.

See also [[familiarity|Familiarity and scheduling]], [[reports]] and [[glossary]].
