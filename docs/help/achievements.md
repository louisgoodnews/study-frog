# Achievements

Achievements are milestones that mark your progress: your first run, a week in a row, a hundred items mastered.
Open **Achievements** in the sidebar.

## The screen

The achievements are grouped by what they measure (**Streaks**, **Runs**, **Ratings**, **Cards**, **Mastery**) and by
stack. Each group shows how many of its achievements are unlocked, for example "Streaks (3 / 7 unlocked)". Inside a
group the unlocked ones come first, with the day each one unlocked; the others show a bar with how far you are, for
example "4 / 7".

Use the buttons at the top to narrow the list: **All**, **Global** (the achievements of the whole vault), or one
stack. The page of a stack also shows how many of its achievements are unlocked; click it to see them.

When you reach one, StudyFrog announces it ("Achievement unlocked: ...") as a message and a notification. Each
achievement unlocks once and is remembered.

## The built-in set

Each kind has several tiers:

- **Streaks** (days in a row on which you review something): 3, 7, 14, 30, 60, 100 and 365 days.
- **Runs** (finished rehearsal runs): 1, 10, 25, 50 and 100.
- **Ratings** (items rated): 50, 250, 1,000 and 5,000; and 20 or 100 items rated Easy.
- **Mastery** (items scheduled 21 days or more ahead): 10, 50, 100, 250 and 500.
- **For every stack separately**: 10, 25, 50 or 100 different items reviewed; 10, 25, 50 or 100 items mastered; and 50
  items rated.

An item counts as **mastered** when its next review is 21 days or more away.

## What counts

Achievements are worked out from your review history. Only runs that update the schedule write reviews, so
practice-only runs count towards "finish a run" but not towards the achievements about ratings, items or streaks.
See [[rehearse]].

## Your own achievements

You can replace the built-in set with your own. Create a file `achievements.toml` in the vault folder:

```toml
[[achievement]]
id = "week"
title = "A week in a row"
description = "Review something on 7 days in a row."
scope = "global"
metric = "streak_days"
threshold = 7
```

- `id`: one word, unique. It is how StudyFrog remembers the unlock.
- `scope`: `global`, or `stack` to unlock separately for every stack.
- `metric`: `runs_finished` (global only), `items_rated`, `easy_ratings`, `items_learned`, `items_mastered` or
  `streak_days`.
- `threshold`: at least 1.

When the file exists it **replaces** the built-in set; an empty file means no achievements. If the file has a
mistake, StudyFrog reports it with its line and uses the built-in set. See [[settings]] for the other settings
files.

## Related pages

[[reports]], [[home]] (your streak), [[faq]].
