---
id: STORY-0188
type: story
title: "Keyboard Shortcut to Focus Search (Ctrl+F and /)"
status: review
epic: EPIC-057
parents: ['EPIC-057']
estimate: 3
labels: [m14, ui, ux]
acceptance_criteria:
  - 'Ctrl+F focuses the search field of the current screen (library, stack page, PDF viewer) and selects its text'
  - '/ focuses the same search field when no text field has keyboard focus'
  - 'The search field''s hover text and its accessible name state the shortcut'
---

## Context
Search is the fastest way to find an item, a stack or a passage in a PDF, but nothing on the keyboard reaches it. The shortcut is also the first thing a keyboard user looks for, and STORY-0169 asks for shortcuts to be shown where they exist.

## Description
- Add one shortcut handler for the screens that have a search field: the library search bar, the stack page's filter and the PDF viewer's "Find in this PDF". Ctrl+F (Cmd+F on macOS) opens and focuses the field of the screen in front.
- Accept `/` as well, when no text field has keyboard focus, so the key works without a modifier.
- Show the shortcut in the field's hover text and accessible name, using the helpers in `studyfrog-ui::view::tips`.
- Make sure the shortcut does not steal keys from text being typed in notes, cards and dialogs.

## Acceptance Criteria
- [x] Ctrl+F focuses the search field of the current screen (library, stack page, PDF viewer) and selects its text
- [x] `/` focuses the same search field when no text field has keyboard focus
- [x] The search field's hover text and its accessible name state the shortcut

## Design Notes
The app has no `/` shortcut today (the stack page shows a literal "/" as a breadcrumb separator), so this adds one. Screen readers must announce the field and the shortcut. Keep the handler in one place (`studyfrog-ui`), not in each view.

## Dependencies
EPIC-057, STORY-0169 (tooltip helpers)

## Implementation (2026-11-21)
`studyfrog-ui::search_focus`: each search field asks `focus_field` whether the shortcut was pressed (Ctrl+F or Cmd+F, or `/` while nothing has the keyboard) and, if so, takes focus and selects its text. Only the field on screen is drawn, so only it can take focus. The hover text and the accessible name ("Find in this PDF (Ctrl+F or /)", "Search titles (Ctrl+F or /)", "Query text (Ctrl+F or /)") state the shortcut. Tests: `studyfrog-ui/tests/pdf_viewer.rs` (Ctrl+F and `/` in the viewer), `tests/library_search.rs` (Ctrl+F in the library).

## Decided
- Ctrl+F on the PDF viewer opens "Find in this PDF", not the library search (decided by the owner).

## Open Questions
None. The two questions were decided; see the implementation notes above.
