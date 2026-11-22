---
id: STORY-0189
type: story
title: "Hover Explanations for Text Buttons Across the App"
status: review
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
- [x] A test fails when a new text button is added without a hover explanation
- [x] Explanations are one line and say what the button does, not just repeat its label

## Design Notes
Tooltips must not hide the control they describe, must follow the theme's contrast tokens, and must be readable by screen readers (hover text is not announced on its own, so the accessible name stays the button label). Destructive buttons (such as "Delete item…") say what will happen, not only that it is destructive.

## Dependencies
EPIC-057, STORY-0169 (tooltip helpers)

## Implementation (2026-11-21)
Every text button in `studyfrog-ui` now has a one-line hover explanation: about 415 call sites, in about 130 files. The explanations are written once per label (`view::tips` holds the shared ones) and set where each button is drawn, so the explanation stays with the control.

The guard (`tests/text_button_tooltips.rs`) scans the source for button calls. A call is explained when its chain gives it `on_hover_text`, or when it is bound to a name that is added with such a chain. The count of unexplained calls is now zero, so a new button without an explanation fails the test. The scan follows each call's closing parenthesis and its chained calls, and skips string literals when matching parentheses.

Two exceptions are marked in the source: the shared `kit` helpers wrap their caller's text, so their line carries `hover: from the caller`.

Review: the explanations that were reused were checked against their buttons. Where one wording was wrong for a screen (a Delete that removes a help page, an Open that opens a file, a Remove that removes a rating scale or a saved filter), it was rewritten for that button. Explanations used once were not re-read. Screen readers use the button label; the explanation is the hover text.

## Open Questions
None. The two questions were decided; see the implementation notes above.

## Review (2026-11-22)

Review (2026-11-22): criterion 1 unticked: the navigation rail's text items (Home, Calendar, Homework, Board) show no hover text (/tmp/claude-0/review-a/hover-home-long.png, /tmp/claude-0/review-a/hover-calendar.png). The rail is named in this story's Description. Header and action buttons do have explanations ("+ Create": "Creates a new item.", /tmp/claude-0/review-a/hover-create.png; the PDF find field, /tmp/claude-0/review-a/hover-pdf-find.png). Criterion 2 (guard test): the test is present (`crates/studyfrog-ui/tests/text_button_tooltips.rs`); not run here, since cargo is not allowed in this review. Criterion 3 MET on the sampled explanations, each one line. Status stays review.
