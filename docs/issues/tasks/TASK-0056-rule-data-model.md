---
id: TASK-0056
type: task
title: "Rule data model"
status: done
epic: EPIC-014
parents: [STORY-0026]
estimate: 3
labels: [m7, achieve]
acceptance_criteria:
  - "`Rule`, `Metric` (runs finished, items rated, easy ratings, items learned, items mastered, streak days), `AchievementScope`"
  - "`achievements.toml` is parsed with line-numbered errors; unknown keys, bad metrics, repeated ids and thresholds of 0 are refused"
  - "A built-in set exists: first run, 10 runs, 7 and 30 day streaks, 20 Easy ratings, 100 mastered, and two per-stack goals"
---

## Context
Rules as data.

## Description
Rule data model.

## Acceptance Criteria
- [x] `Rule`, `Metric` (runs finished, items rated, easy ratings, items learned, items mastered, streak days), `AchievementScope`
- [x] `achievements.toml` is parsed with line-numbered errors; unknown keys, bad metrics, repeated ids and thresholds of 0 are refused
- [x] A built-in set exists: first run, 10 runs, 7 and 30 day streaks, 20 Easy ratings, 100 mastered, and two per-stack goals

## Design Notes
`core::achievement` (types), `config::achievements` (TOML), `achieve::defaults`. Mastered means scheduled 21 days or more ahead (FSRS stability or SM-2 interval). Only schedule-updating runs write reviews, so practice runs do not count towards review-based achievements.

## Dependencies
None.

## Open Questions
None.
