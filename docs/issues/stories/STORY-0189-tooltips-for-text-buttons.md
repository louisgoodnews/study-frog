---
id: STORY-0189
type: story
title: "Hover Explanations for Text Buttons Across the App"
status: backlog
epic: EPIC-057
parents: ['EPIC-057']
estimate: 5
labels: [m14, ui, ux]
acceptance_criteria:
  - 'Every text button on the navigation, header, action tray and dialog screens has a hover explanation, with its keyboard shortcut where one exists'
  - 'A test fails when a new text button is added without a hover explanation'
  - 'Explanations are short (one line) and state what the button does, not just repeat its label'
---

## Context
STORY-0169 gave the icon buttons explanations on hover, and the rating and difficulty buttons their shortcuts. Most of the app's buttons are text buttons ("Back", "Preview", "Choose file…", "Revert", "Assign keys…"), and most of them still show nothing beyond their label. The app has about 330 button call sites, so the gap is wide.

## Description
- Audit every text button in `studyfrog-ui` (`ui.button`, `kit::primary_button`, `kit::quiet_button`, `menu_button` and `Button::new`) and sort them into screens that need an explanation now and those that can wait.
- Give each button a one-line explanation through `studyfrog-ui::view::tips`, next to where the button is drawn, so the text stays with the control.
- Add a guard test that lists the button labels that have an explanation, so a new button without one is noticed.
- Start with the navigation rail, header bars, the rehearsal action tray, the item editor and the import and export dialog, then the rest.

## Acceptance Criteria
- [ ] Every text button on the navigation, header, action tray and dialog screens has a hover explanation, with its keyboard shortcut where one exists
- [ ] A test fails when a new text button is added without a hover explanation
- [ ] Explanations are one line and say what the button does, not just repeat its label

## Design Notes
Tooltips must not hide the control they describe, must follow the theme's contrast tokens, and must be readable by screen readers (hover text is not announced on its own, so the accessible name stays the button label). Destructive buttons (such as "Delete item…") say what will happen, not only that it is destructive.

## Dependencies
EPIC-057, STORY-0169 (tooltip helpers)

## Implementation (2026-11-21)
`studyfrog-ui/tests/text_button_tooltips.rs` scans every source file for button calls (`.button(`, `.menu_button(`, `primary_button(`, `quiet_button(`, `Button::new(`, `.small_button(`). A call is explained when its statement gives it `on_hover_text` or a `view::tips` text. The count of unexplained calls is a ratchet: `BASELINE` is 415 and may only go down. A test checks that the scanner itself finds a plain button.

Decided: the guard scans the source rather than keeping a list of labels (owner's choice). Limits: a button whose hover text is set on a later statement is counted as unexplained, and labels built at run time are judged by their call site, not their text.

Still open: the 415 buttons. Lower `BASELINE` as each one is explained, starting with the navigation, header bars, action trays and dialogs.

## Open Questions
None. The two questions were decided; see the implementation notes above.
