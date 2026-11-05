---
id: STORY-0086
type: story
title: Expanded achievement catalog and milestone tiers
status: backlog
epic: EPIC-029
parents: [EPIC-029]
estimate: 3
labels: [m12, achieve]
acceptance_criteria:
  - "Default achievement rules in studyfrog-achieve include tiered milestones for streaks, runs, ratings, and mastered items"
  - "Per-stack default rules provide graduated goals for learned items, mastered cards, and ratings within each stack"
  - "Evaluator accurately tracks and unlocks new tiers from existing review history without invalidating previous unlocks"
---

## Context
Currently, `studyfrog-achieve/src/defaults.rs` defines only 8 default achievements. Once an active learner reaches a 30-day streak or masters 100 items, there are no further goals to strive for. Furthermore, per-stack achievements only cover 2 static milestones (25 items learned, 25 items mastered), leaving smaller stacks out of reach or larger stacks without continuing goals.

## Description
Expand the built-in achievement rules in `studyfrog-achieve`:
- Streak tiers:
  - `streak-3`: "Getting into the rhythm" (3 days)
  - `streak-7`: "A week in a row" (7 days) [existing]
  - `streak-14`: "Two weeks strong" (14 days)
  - `streak-30`: "A month in a row" (30 days) [existing]
  - `streak-60`: "Two months of habit" (60 days)
  - `streak-100`: "Centurion" (100 days)
  - `streak-365`: "Year of study" (365 days)
- Rehearsal run tiers:
  - `first-run`: "First run" (1 run) [existing]
  - `ten-runs`: "Regular" (10 runs) [existing]
  - `runs-25`: "Dedicated" (25 runs)
  - `runs-50`: "Half-century" (50 runs)
  - `runs-100`: "Century club" (100 runs)
- Ratings & practice volume:
  - `rated-50`: "Warming up" (50 items rated)
  - `rated-250`: "Deep practice" (250 items rated)
  - `rated-1000`: "Knowledge worker" (1,000 items rated)
  - `rated-5000`: "Study machine" (5,000 items rated)
  - `easy-20`: "Easy does it" (20 easy ratings) [existing]
  - `easy-100`: "Effortless recall" (100 easy ratings)
- Mastery tiers (Global):
  - `mastered-10`: "First foothold" (10 items mastered)
  - `mastered-50`: "Growing foundation" (50 items mastered)
  - `mastered-100`: "Hundred mastered" (100 items mastered) [existing]
  - `mastered-250`: "Scholar" (250 items mastered)
  - `mastered-500`: "Grandmaster" (500 items mastered)
- Per-stack tiers:
  - `stack-learned-10`: "First steps" (10 items learned in a stack)
  - `stack-learned-25`: "Getting started" (25 items learned in a stack) [existing]
  - `stack-learned-50`: "Familiar territory" (50 items learned in a stack)
  - `stack-learned-100`: "Comprehensive grasp" (100 items learned in a stack)
  - `stack-mastered-10`: "First mastery" (10 items mastered in a stack)
  - `stack-mastered-25`: "Stack master" (25 items mastered in a stack) [existing]
  - `stack-mastered-50`: "Deep deck fluency" (50 items mastered in a stack)
  - `stack-mastered-100`: "Stack perfection" (100 items mastered in a stack)
  - `stack-rated-50`: "Deck dedication" (50 ratings in a stack)

## Acceptance Criteria
- [ ] Default achievement rules in studyfrog-achieve include tiered milestones for streaks, runs, ratings, and mastered items
- [ ] Per-stack default rules provide graduated goals for learned items, mastered cards, and ratings within each stack
- [ ] Evaluator accurately tracks and unlocks new tiers from existing review history without invalidating previous unlocks

## Design Notes
- Files: `crates/studyfrog-achieve/src/defaults.rs`, `crates/studyfrog-achieve/tests/achieve.rs`.
- Ensure IDs of existing rules (`first-run`, `ten-runs`, `streak-7`, `streak-30`, `easy-20`, `mastered-100`, `stack-learned-25`, `stack-mastered-25`) are preserved verbatim to maintain unlock history compatibility.
- Keep `defaults.rs` concise and ensure line count stays well within the 250-line limit (rule P2).
- Release target: version 0.5.0 (Phase M12).

## Dependencies
EPIC-029.

## Open Questions
None.
