# Reports

Reports show how your learning is going: what you remember, how fast you answer, how familiar you are with your
items, and what is coming up. Open **Reports** in the sidebar. The reports need a few reviews first; until then
the screen says "There are no reviews yet."

**Export as CSV** (top of the screen) writes the numbers of the reports into one CSV table in the `exports` folder of
your vault, as `studyfrog-report-<vault>-<date>.csv`, and tells you where; **Save CSV to...** asks for a folder first.
Each row has a section (summary, stack, retention, latency, forecast, familiarity or runs), a subject, a metric and a
value, so a spreadsheet can pivot it. The summary holds the reviews, the items learned (reviewed at least once) and
mastered (scheduled three weeks or more ahead) and your streak of days with a review; each stack has its items, the
items due now, learned and mastered, its reviews, the average interval in days and its retention (share of reviews
not rated Again).

The screen has these tabs: **Performance**, **Memory and forecast**, **Familiarity**, **Self-assessment**, **Runs**,
**Habits**, **Leeches** and **Workload**.

## Performance

Choose what to look at and the period, then drill down by clicking:

- **Scope**: the whole vault, a stack, a subject (**By subject**) or one item. Click a stack to see the stacks and
  items **Inside** it; **Open the item** opens an item's page.
- **Period**: **All time**, **Last 30 days** or **Last 7 days**.

For the chosen scope and period you see the number of **Reviews**, **Items reviewed**, the share rated **Good or
easy**, the **Median time to answer** and **Time answering**, bars for **How the reviews were rated** (Again, Hard,
Good, Easy), a chart of the **Last 30 days**, and the **Last review**.

## Memory and forecast

- **Retention**: how often an item was recalled, by the time since its previous review (1 day, 2 days, 3-4 days,
  and so on up to 91+ days). A review counts as recalled unless you rated it Again.
- **Time to answer**: the median and the times 90 % and 95 % of your answers stayed under, a histogram, and the
  same split by the mark you gave the item (easy, medium, difficult, not marked). Paused time is left out.
- **Reviews ahead**: how many reviews to expect on each of the next 30 days, and per week. Overdue items count
  today. It is an expectation from your history, not a promise.
- **True retention**: recall of mature items (stability of 21 days or more) reviewed within 20 % of their interval,
  next to young, mature, early, on-time and overdue reviews, each with what FSRS expected; your memory half-life for
  a typical item; and **Forgetting curves by stability**: for items under 7, 7-30, 30-90 and 90+ days of stability,
  measured recall against the FSRS prediction by the time since the review relative to the stability (t/S).
- **Checked on**: how well the forecast did when tested on the past. With at least 56 days of your own history it is
  checked on yours; otherwise on a simulated learner, and the screen says so. On simulated histories the weekly
  totals were off by about 10 %.

## Familiarity

**How well you know your items**: how your items spread over the familiarity scale (0-9 %, 10-19 %, and so on), and
how many have no history yet. **By stack** and **By subject** show the familiarity of each, with its three least
familiar items and what drives each value. See [[familiarity]].

## Runs

Every rehearsal run with its date, mode (Flip cards or Type answers), how many items were rated and skipped, whether
it finished or was stopped early, its pacing (no clock, countdown or count up), the planned time of a countdown, the
time used and the answers per minute. **Show** filters the list: **All runs**, **Flip cards** or **Type answers**.

## Habits

- **When you remember best**: recall and time to answer by the local hour of the review, the two hours where your
  recall is highest for its time (and the lowest), and by weekday. **Reviews an hour needs** (5, 10, 20, 50) sets how
  many reviews an hour must have before it is recommended.
- **Within a run**: recall and time to reveal by minutes into a run (pauses left out) and by the item's place in it,
  and where accuracy starts to fall or answers slow down: a suggested run length. During a run a quiet line says when
  an answer took far longer than the run's usual; switch it off with **Don't show this** or in Settings > Rehearsal.

## Leeches

Items that keep failing (4 lapses or more, 3 hard or forgotten in a row, 3 times the stack's usual time, or many
reviews and still a short memory), with the reasons, and **Items that get confused**: pairs forgotten close together
that share words, a tag or a link. **Edit or split** opens the item, **Pause** takes it out of the schedule, **Drill**
starts a practice run of exactly those items (a pair side by side) that does not change the schedule.

## Workload

- **Workload ahead**: reviews or **Study minutes** for **30**, **90** or **365 days**, from 16 simulated futures of
  every card's FSRS or SM-2 state with your ratings, missed days, new cards a day and the time reviews take in each
  stack; by stage (new, learning, young, mature) with a typical value and a low-to-busy range.
- **Exam pacer**: for each stack with an exam or due date ahead, the new cards a day to start, the reviews and
  minutes a day, and the share of cards that would be at 90 % recall on the day; the last three days are kept for
  consolidation.
- **Workload against desired retention**: each target from 0.75 to 0.97 simulated for 180 days: minutes a day, cards
  remembered and cards per minute; the least-work target and the recommended one. **Apply recommended retention**
  writes it into `studyfrog.toml`.

## A stack's reports

The **Reports** tab of a [[stack-page|stack page]] shows the stack's familiarity and the items it knows least, its
exam pacer, its leeches and confused pairs, and its own workload-against-retention curve, whose **Apply** button writes
the stack's `_stack.toml`; **Open all reports** comes here. The stack's **Summary** also shows its exam pacer.

## Good to know

- Practice-only runs do not write reviews, so they do not count in Performance, Retention or the forecast. Their
  answer times do count in Time to answer.
- Reports are computed when you open the screen. They take a few milliseconds even with 100 000 reviews; the
  simulations take up to about 50 ms each for 5 000 cards. Every simulation uses fixed seeds, so the same history
  gives the same numbers.

## Related pages

[[achievements]], [[rehearse]], [[faq]], [[glossary]].
