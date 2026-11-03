# Achievements

Achievements are milestones that mark your progress: your first run, a week in a row, a hundred items mastered.
Open **Achievements** in the sidebar.

## The screen

- **Unlocked**: the achievements you have reached, with the day each one unlocked.
- **In progress**: the others, each with a bar showing how far you are, for example "4 / 7".

When you reach one, StudyFrog announces it ("Achievement unlocked: ...") as a message and a notification. Each
achievement unlocks once and is remembered.

## The built-in set

- **First run**: finish a rehearsal run.
- **Regular**: finish 10 rehearsal runs.
- **A week in a row**: review something on 7 days in a row.
- **A month in a row**: review something on 30 days in a row.
- **Easy does it**: rate 20 items Easy.
- **Hundred mastered**: have 100 items scheduled 21 days or more ahead.
- **Getting started**: review 25 different items of a stack (separately for every stack).
- **Stack master**: have 25 items of a stack scheduled 21 days or more ahead (separately for every stack).

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
