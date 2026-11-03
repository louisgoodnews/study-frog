# Familiarity and scheduling

StudyFrog decides when you should see each item again. This page explains how: the spaced-repetition schedule,
the familiarity value, and how the two work together.

## Spaced repetition

Each time you rate an item, StudyFrog works out when you are about to forget it and makes it due then. Easy items
come back after longer gaps, hard ones sooner. A rating of Again brings an item back soon.

StudyFrog has two methods:

- **FSRS** (the default) models your memory of each item with three numbers: how hard the item is for you
  (difficulty), how long your memory of it lasts (stability), and how likely you are to recall it right now
  (retrievability). It schedules each review for a **target retention**: with the default 0.9 you should recall
  about 90 % of your reviews. A higher target means more reviews and better recall.
- **SM-2**, the classic method of older flashcard programs, as a simpler choice.

Choose the method for the vault in `studyfrog.toml` and for a stack in its `_stack.toml`. A stack can also limit how
many new items and reviews it offers per day. See [[settings]]. When a stack switches method, its items keep their
due dates; the next review starts over in the new method.

## Priority, difficulty and status

- **Priority** decides what goes first when there is more due than your daily limit allows. A high priority makes an
  item due 12 hours early, the highest 24 hours early; the lowest 12 hours late.
- **Difficulty** (easy, medium, difficult) stands in for the memory difficulty of an item you have never reviewed.
- Items and stacks that are **Complete** or **On hold** are not scheduled. See [[statuses|Item statuses]].
- Stacks whose due date has passed are not suggested any more.

## Familiarity

Familiarity is **how familiar you are with an item**, from 0 to 100 %. 100 % means you know it very well and need
not rehearse it soon. It is not a chance of recall; it is a summary of your history with the item.

It changes like this:

- **Easy** raises it a lot, **Good** a fair amount.
- **Hard** lowers it by a quarter, **Again** by half.
- Each **view** of the item page raises it a little (at most one view per hour counts; views under a second do not).
- **Speed** matters: an answer read faster than your reading speed predicts raises it more; a slow one less.
- Your **mark** matters: an item marked easy gains a bit more, one marked difficult less.
- **Close to an exam** or due date of the item's stack, gains count for less: in full from 14 days before, half on
  the day itself.
- **Without contact** it falls slowly: a year without contact halves it. It does not fall while the item is
  Complete, nor after the exam date of its stack has passed.

A **note** has no ratings, only views, so its familiarity stops at 80 %: reading a note does not show that you know
it. A **stack's** familiarity is the average of its items, weighted by priority. With few reviews behind it, the
value is marked "few reviews".

## Where you see it

- The item page's **Familiarity** card, with what drives the value and its course over time. See [[item-page]].
- **Needs attention** on [[home|Home]]: the five least familiar items.
- The **Familiarity** tab of [[reports]], by stack and by subject, and a stack's Reports tab.
- The `familiarity` column and field in the [[library]] and the [[query-language|query language]].
- Run setup: the order **Least familiar first** and **Only items I am less familiar with than a level**. See
  [[rehearse]].

## How familiarity changes the schedule

After each rating, StudyFrog compares the item's familiarity with what FSRS expects from its stability. If you know
the item better than FSRS thinks, the next gap is stretched; if worse, it is shortened. The factor is between a
quarter and double. It starts after the third review, never stretches a gap past an exam or due date of the stack,
and never changes FSRS's own memory numbers. With SM-2 the due date is adjusted the same way, while SM-2's own
intervals keep growing as usual.

- With a daily limit, the least familiar due items are chosen first.
- `familiarity_threshold` in `studyfrog.toml` leaves out due items at or above that familiarity.
- `familiarity_strength` (default 2, from 0 to 10) sets how strongly familiarity changes the gaps; 0 switches it
  off. `familiarity = false` in a `_stack.toml` switches it off for one stack.

## Does it help you?

Settings, **Study and scheduling**, **Does familiarity help?** replays your own review history and tells you whether
familiarity predicts your recall better than plain FSRS, which values fit you best, and how your number of reviews
would change. It needs enough reviews (with 1 000 or more it also fits the values). It never switches anything
itself.

## Marks do not change the schedule

The easy, medium and difficult marks you give during a run affect familiarity and the rating suggestions, but they
never change the schedule by themselves. Your rating does.

See also [[glossary]] and [[faq]].
