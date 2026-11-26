---
id: STORY-0231
type: story
title: "Swappable grading for multiple choice and multiple select: all-or-nothing, Zapechelnyuk, or count-based partial credit"
status: done
epic: EPIC-072
parents: ['EPIC-072']
estimate: 5
labels: [m14, rehearsal, ui]
acceptance_criteria:
  - "Three grading schemes exist as pure functions over the chosen and the correct options: all-or-nothing (the current rule), Zapechelnyuk (2015) and count-based partial credit"
  - "Zapechelnyuk: a chosen set of k options earns 1/k of the maximum if at least one chosen option is correct, else zero (the published rule, as the owner chose: credit 1/k when any correct option is chosen)"
  - "Count-based partial: (correct chosen minus wrong chosen) divided by the number of correct options, never below zero"
  - "A global setting chooses the scheme for all choice items; all-or-nothing is the default and keeps today's results"
  - "The grade shown after a submit states the scheme and the fraction credited; the recorded score and the suggested rating follow the chosen scheme"
  - "Unit tests cover each scheme on single choice and on multiple select, including the boundary cases (nothing chosen, everything chosen, one wrong, all correct)"
---

## Context
Choice items are graded all-or-nothing (STORY-0221, reusing the mock exam rules). The owner asked (2026-11-25) for the
grading to be swappable: all-or-nothing, the Zapechelnyuk rule (Economics Letters 2015, "An axiomatization of multiple-choice
test scoring"), or count-based partial credit.

## Description
Put the three schemes behind one pure function that returns a fraction in 0 to 1. The rehearsal engine multiplies the
item's maximum by the fraction; the suggested rating follows the fraction (full credit: Good; partial: Hard; none: Again).
The scheme is chosen by a global setting.

## Acceptance Criteria
- [x] Three pure grading schemes
- [x] Zapechelnyuk rule as published, applied to several correct options as chosen (1/k when any correct option is chosen)
- [x] Count-based partial credit, never below zero
- [x] Global setting; all-or-nothing by default
- [x] Grade text states the scheme and the fraction; recorded score and rating follow it
- [x] Unit tests for each scheme, single and multiple, at the boundaries

## Design Notes
- Zapechelnyuk, per the owner's choice: credit 1/k when any correct option is chosen, k being the number of options chosen.
  Picking more options dilutes credit. Nothing chosen earns zero.
- Count-based: (correct chosen - wrong chosen) / correct total, clamped to 0 and 1.
- Sources: Zapechelnyuk, A. (2015), Economics Letters 132, 24-27 (https://econpapers.repec.org/RePEc:eee:ecolet:v:132:y:2015:i:c:p:24-27); Persson (2023), METRON (https://ideas.repec.org/a/spr/metron/v81y2023i2d10.1007_s40300-022-00237-w.html).
- The setting lives with the other app settings in `studyfrog-core` (AppSettings), so it is per user, not per vault. Store it with the existing settings writer.

## Dependencies
- STORY-0220 (parser), STORY-0221 (grading path), STORY-0222 (rendering shows the grade).

## Open Questions
- None. The scheme is a global setting (decided 2026-11-25).
