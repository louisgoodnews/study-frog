---
id: STORY-0087
type: story
title: Per-stack achievement UI and filtering
status: review
epic: EPIC-029
parents: [EPIC-029]
estimate: 3
labels: [m12, achieve, ui]
acceptance_criteria:
  - "The achievements screen provides a filter control to view All, Global-only, or achievements for a selected stack"
  - "Per-stack achievements are visually grouped by stack or clearly badged rather than mingling into one long unfilterable list"
  - "The stack browser screen displays a concise achievement summary badge indicating unlocked and total available achievements for the selected stack"
---

## Context
When a user has multiple stacks (decks) in their vault, per-stack achievements duplicate across each stack (e.g. `Getting started (French)`, `Getting started (Biology)`, `Stack master (French)`). In the current Achievements view, all of these appear in a single vertical list without filtering, causing visual clutter and making it difficult to assess progress on a particular subject.

## Description
Enhance the achievements presentation in `studyfrog-ui` and `studyfrog-service`:
1. Achievements Screen Filtering:
   - Provide a selector (dropdown or horizontal pill bar) in `crates/studyfrog-ui/src/view/achievements.rs` allowing users to filter achievements:
     - `All`: View everything.
     - `Global`: Show only global achievements.
     - `[Stack Name]`: Show achievements specific to the selected stack.
2. Grouping and Visual Polish:
   - When viewing `All` or stack achievements, group entries by category (e.g. Streaks, Runs, Cards, Mastery) or by stack, preventing repetitive cards.
   - Display progress bars with clear fraction formatting (`X / Y`) and stack badge tags.
3. Stack Browser Integration:
   - On the Stack Browser view (`crates/studyfrog-ui/src/view/browser.rs`), display a compact achievement counter or status pill (e.g. `🏆 3 / 8 achievements unlocked`) for the currently selected stack.
   - Clicking or hovering reveals the stack's achievement progress or links to the filtered achievements view.

## Acceptance Criteria
- [x] The achievements screen provides a filter control to view All, Global-only, or achievements for a selected stack
- [x] Per-stack achievements are visually grouped by stack or clearly badged rather than mingling into one long unfilterable list
- [x] The stack browser screen displays a concise achievement summary badge indicating unlocked and total available achievements for the selected stack

## Design Notes
- Touchpoints:
  - `crates/studyfrog-service/src/achievements.rs`: Add helper queries to filter report entries by `Option<StackId>` or scope.
  - `crates/studyfrog-ui/src/view/achievements.rs`: Add filter state in `State` (e.g., `achievement_filter: Option<Option<String>>`), render stack selector filter bar, and render filtered subsets.
  - `crates/studyfrog-ui/src/view/browser.rs`: Query and display stack achievement count for the active stack.
- Comply with all repository architecture rules: free functions only (no inherent impls, rule P1), and all files under 250 lines (rule P2).
- Release target: version 0.5.0 (Phase M12).

## Progress (2026-11-08)
Built and tested headless: the filter (All, Global, one stack), groups by category and by stack with the unlocked ones first, `X / Y` bars, and the summary link on a stack's page (which asks for the report once if it is not loaded). Not looked at in a real window.

## Dependencies
EPIC-029, STORY-0086.

## Open Questions
None.
