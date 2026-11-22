---
id: STORY-0169
type: story
title: "Contextual Tooltips, Hover Hints, and Keyboard Shortcut Badges"
status: review
epic: EPIC-057
parents: ['EPIC-057']
estimate: 3
labels: [m14, ui, ux]
acceptance_criteria:
  - 'Every icon button on the navigation rail, header bars, and action trays exhibits an explanatory tooltip on hover'
  - 'Tooltips display associated keyboard shortcuts where available (e.g. F1, Space, 1-4, Ctrl+F)'
  - 'Tooltip delays and styling conform to theme token contrast rules and never obscure clicked controls'
---

## Context
Navigation feels difficult and unintuitive because action icons and buttons lack hover explanations or keyboard shortcut cues.

## Description
Conduct a systematic audit and implementation of hover tooltips across the UI:
- Add `on_hover_text` or rich `on_hover_ui` badges to all navigation rail icons (Home, Stacks, Calendar, Rehearsal, Exam, Graph, Scripts, Settings).
- Include hotkey badges in tooltips for rehearsal grading buttons (1: Again, 2: Hard, 3: Good, 4: Easy), flip card (Space), and search (`/`).
- Provide brief 1-line explanations for complex buttons so users immediately understand their purpose before clicking.

## Acceptance Criteria
- [x] Every icon button on the navigation rail, header bars, and action trays exhibits an explanatory tooltip on hover
- [x] Tooltips display associated keyboard shortcuts where available (e.g. F1, Space, 1-4, Ctrl+F)
- [ ] Tooltips conform to theme token contrast rules and never obscure clicked controls

## Implementation so far (2026-11-21)
`studyfrog-ui::view::tips` holds the tooltip text: each icon button's name and a one-line explanation (`for_name`, used by `icon_button` for all of the rail, header and dialog icons), and the shortcut on its own line for the rating buttons (1 to 4), the difficulty marks (E, M, D), Show answer (Space) and Skip (S). Tests: `studyfrog-ui/tests/tooltips.rs`.

Not done: the search field has no `/` shortcut in the app, so none is shown for it. Text buttons outside the rehearsal screen have no tooltips yet. Placement and contrast are egui's defaults and the theme's colours; they have not been checked on screen, so the third criterion is not ticked.

## Design Notes
Implement consistent helper functions in `studyfrog-ui::view::widget` for formatted tooltips with shortcut badges.

## Dependencies
EPIC-057

## Implementation (2026-11-22)
Built: the tooltip delay is a named token, `view::tips::DELAY_SECS` (0.5 s), and the theme sets it (`theme_apply`). Test: `tests/tooltips.rs::the_theme_sets_the_tooltip_delay`. A new test, `a_click_hides_the_tooltip_so_it_never_stays_over_the_control`, checks that a tooltip shows while the pointer rests on a button and is gone after a click. egui 0.37 hides a tooltip when the control is clicked or the pointer moves on (`containers/tooltip.rs`, `should_show_tooltip`). The click test opens the columns menu too, which also hides tooltips; the check is what the user sees.

Corrections to the notes above: (1) the search shortcut exists now. STORY-0188 adds `/` and Ctrl+F and puts the shortcut in the search field's hover text, so the "no `/` shortcut" gap is closed. (2) Text buttons outside rehearsal are covered: `tests/text_button_tooltips.rs` scans every text button call in the UI source and fails when one has no hover text.

Contrast: tooltips take the window fill (the surface token) and the text token (`theme.rs`, `visuals`). `tests/appearance_palette.rs` checks text on the surface at 4.5:1 or more for every palette, light and dark, normal and high contrast. That covers tooltips through the shared tokens; no test is specific to tooltips.

Still open, needs a person at a window: where a tooltip lands next to neighbouring controls (egui places it below the control with a 4 px gap, so it can sit over the control below), and whether the text reads well on each theme. The third criterion stays unticked until that is checked.

## Open Questions
None.
