---
id: EPIC-029
type: epic
title: "Expanded achievements and per-stack progression"
status: done
parents: []
labels: [m12, achieve, ui]
acceptance_criteria:
  - "The default achievement catalog provides tiered milestones across runs, streaks, ratings, and mastery"
  - "Per-stack achievements can be filtered and viewed in the achievements screen and on individual stack views"
  - "Vault review history correctly evaluates all new global and per-stack rules without regressing existing unlocks"
---

## Context
StudyFrog includes a foundation for achievements in `studyfrog-achieve` and `studyfrog-ui`, but currently only ships 8 basic default rules (6 global, 2 per-stack) and renders all achievements into an unorganized single list. Users who study across multiple diverse decks want richer progression milestones (tiered accomplishments for consistent habits, review volume, and mastery) as well as the ability to focus on achievements for specific stacks rather than seeing an undifferentiated feed.

## Description
Expand the achievements system in StudyFrog version 0.5.0:
1. Provide a comprehensive catalog of default achievements featuring milestone tiers (e.g. Bronze, Silver, Gold, Platinum) across key study habits:
   - Consistency: Streak milestones (3, 7, 14, 30, 60, 100, 365 days).
   - Rehearsal dedication: Finished runs (1, 10, 25, 50, 100, 250 runs).
   - Card reviews: Ratings given (50, 250, 1000, 5000 ratings).
   - Mastery: Items mastered at 21+ days interval (10, 50, 100, 250, 500 cards).
2. Expand per-stack achievements:
   - Provide tiered learning and mastery milestones per stack (e.g., 10, 25, 50, 100 items learned; 10, 25, 50, 100 items mastered).
   - Add stack-specific review count milestones.
3. Enhance the achievements user experience:
   - Add stack filtering / tab controls to the Achievements screen (`All`, `Global`, or pick a specific `Stack`).
   - Integrate stack achievement progress badges directly into the Stack Browser and Rehearsal Summary views.
   - Maintain full backwards compatibility with historical `unlocked.toml` records.

## Acceptance Criteria
- [x] The default achievement catalog provides tiered milestones across runs, streaks, ratings, and mastery
- [x] Per-stack achievements can be filtered and viewed in the achievements screen and on individual stack views
- [x] Vault review history correctly evaluates all new global and per-stack rules without regressing existing unlocks

## Design Notes
Child issues planned for version 0.5.0:
- STORY-0086: Expanded achievement catalog and milestone tiers
- STORY-0087: Per-stack achievement UI and filtering

Crate touchpoints:
- `crates/studyfrog-core/src/achievement.rs`: Existing metrics and scopes are well-designed and support both global and stack calculations.
- `crates/studyfrog-achieve/src/defaults.rs`: Expand `default_rules()` with structured tier progressions.
- `crates/studyfrog-service/src/achievements.rs`: Rule evaluation and report generation; supports filtering by stack.
- `crates/studyfrog-ui/src/view/achievements.rs`: UI layout improvements, stack selector filter, and grouped sections.
- `crates/studyfrog-ui/src/view/browser.rs`: Stack overview achievement badge/summary.

## Dependencies
EPIC-014 (M7 foundation).

## Open Questions
None.
