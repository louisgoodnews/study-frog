---
id: STORY-0068
type: story
title: Diary for self-reported progress
status: done
epic: EPIC-024
parents: [EPIC-024]
estimate: 8
labels: [m11, ui, core, md, report]
acceptance_criteria:
  - "The user can write diary entries with ratings of performance, success, confidence, effort, focus, mood, stress, energy, motivation, understanding, satisfaction and sleep and free text, and link them to stacks, items, runs and exams"
  - "Entries are Markdown files in the vault, listed by day and searchable"
  - "A report compares the user's self-assessment with measured results such as accuracy and familiarity"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Let the user record how studying feels and how it went, in their own words, and see whether their impressions match the numbers.

## Acceptance Criteria
- [x] The user can write diary entries with ratings of performance, success, confidence, effort, focus, mood, stress, energy, motivation, understanding, satisfaction and sleep and free text, and link them to stacks, items, runs and exams
- [x] Entries are Markdown files in the vault, listed by day and searchable
- [x] A report compares the user's self-assessment with measured results such as accuracy and familiarity

## Design Notes
An entry is a Markdown file `diary/YYYY-MM-DD.md` (several per day are allowed with a suffix) with front-matter for the ratings (0 to 4 scales with named ends, so 'how well did it go' is answered the same way each time; the owner asked for 'everything' on 2026-11-02, so the set is performance, success, confidence, effort, focus, mood, stress, energy, motivation, understanding, satisfaction and sleep, plus scales the user defines themself; Settings chooses which ones the form shows, and every one is optional), also study minutes and a few free tags. Entries may carry pictures, stored in the vault's assets folder under the same rules as pictures in notes, the linked stacks, items, runs and exams, and a free-text body in the same editor as notes. It lives in the vault so it is backed up, searched and exported like any note, and it is never sent anywhere. After a rehearsal run the summary offers a short prompt ('How did that go?') that is skipped with one key and never blocks. The Diary screen lists entries by day with a month view and a filter; the Home page can show 'no entry yet today' as a quiet nudge that can be switched off. The report puts self-rated performance next to measured accuracy and familiarity over time, which shows whether the user over- or underrates themselves (calibration); it states how many entries it rests on. Ratings never change a schedule.

Tasks: TASK-0170, TASK-0171, TASK-0172, TASK-0173, TASK-0174.

## Dependencies
The comparison report needs STORY-0058 for familiarity; the rest stands alone.

## Open Questions
Answered by the owner on 2026-11-02: all of the ratings, so the form shows the ones the user turns on. Entries may carry photos and pictures (the owner agreed on 2026-11-02); they follow the picture rules of notes (STORY-0044).
