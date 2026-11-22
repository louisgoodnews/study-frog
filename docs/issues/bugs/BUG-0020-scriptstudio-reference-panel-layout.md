---
id: BUG-0020
type: bug
title: "ScriptStudio reference panel is squeezed and its topics are not shown"
status: done
estimate: 3
labels: [m14, ui]
acceptance_criteria:
  - "With the FrogScript reference open beside the ScriptStudio editor, the editor text, the reference search and the output pane are all visible at 1280x800 and at 1600x900"
  - "The reference lists every category heading and its topics, and the cookbook's Insert into editor and Run in console buttons are visible and clickable"
---

## Context
Found while checking EPIC-058 in the running app (2026-11-22). The reference opens beside the editor, but the layout breaks, whether it is opened from the Console's "Reference" button or from the new "FrogScript reference" button on the Help screen (the same panel):
- the code editor's text is drawn in a column about 30 px wide at the right edge of the editor area, one character per line, so the script cannot be read;
- the reference's search box is shown, but the topic list and the cookbook below it are blank, even after scrolling;
- the reference runs off the right edge of the window;
- the output pane below the editor is pushed out of the window, so the console output cannot be seen.

With the reference hidden, the editor and output are laid out correctly (the same window, screenshot 04). The fault is in the layout of the reference panel beside the editor, not in its content, which the tests in `studyfrog-ui/tests/studio_reference.rs` cover.

Screenshots (`/tmp/claude-0/epic58/`): `03-help-to-reference.png` (Help button), `05-console-reference-button.png` (Console button, 1280x800), `06-console-reference-1600.png` and `07-console-reference-settled.png` (1600x900, after settling), `08-reference-scrolled.png` (after scrolling the panel), `04-console-after-hide.png` (the same window with the reference hidden).

## Description
Find why `studio_editor` in `crates/studyfrog-ui/src/view/console.rs` gives the editor and the reference the widths and heights it does (`horizontal_top` with two `allocate_ui` calls of 0.62 and 0.36 of the width, and the height left over). Make the editor keep a readable width, the reference scroll and show its topics, and the output pane keep its place below. Check at 1280x800 and 1600x900 in the window with `ctl screenshot`.

## Acceptance Criteria
- [x] With the FrogScript reference open beside the ScriptStudio editor, the editor text, the reference search and the output pane are all visible at 1280x800 and at 1600x900
- [x] The reference lists every category heading and its topics, and the cookbook's Insert into editor and Run in console buttons are visible and clickable

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs` (`studio_editor`, the editor and reference columns), `crates/studyfrog-ui/src/view/studio_reference.rs`, `crates/studyfrog-ui/src/view/console/code.rs`.
- Guardrails: P4 (nesting < 3) and P6 (files <= 250 lines).

## Dependencies
- EPIC-058

## Open Questions
- None.

## Fix (2026-11-22)
Cause: `studio_editor` in `crates/studyfrog-ui/src/view/console.rs` gave the editor and the reference the whole remaining height (`available_height()`), so the output pane below was pushed out of the window. It also split the width with two `allocate_ui` calls at 0.62 and 0.36 of the width, which left the reference's content about 30 px wide and the editor text in a column of 24 to 30 px.

Change: the editor and the reference share a band of 60% of the available height (at least 260 px). In the band the reference is a resizable right panel (`egui::Panel::right`, 38% of the width at first, 300 px up to the width less 300 px), and the editor is the central panel. The output pane keeps the rest of the window. The reference's own scroll area, search, Insert into editor and Run in console are unchanged, and so is the wording.

Tests: `crates/studyfrog-ui/tests/studio_reference_layout.rs` at 960x640, 1280x800 and 1600x900: `the_editor_keeps_a_usable_width_beside_the_reference_at_each_size`, `the_reference_lists_every_category_topic_and_cookbook_button_in_the_window` and `the_output_pane_stays_in_the_window_with_the_reference_open`. On the defective code the first failed with the editor 24 px wide, the second with the topic "from" outside the window and the third with the output pane pushed out.

Verified in the running app (2026-11-22; a copy of the binary on display :90, vault `/tmp/claude-0/wh/vault`):
- Before (the defective build): `/tmp/claude-0/wiki/before-reference-1280.png` (Console "Reference", 1280x800), `/tmp/claude-0/wiki/before-reference-1600.png` (1600x900), `/tmp/claude-0/wiki/before-help-to-reference-1280.png` and `/tmp/claude-0/wiki/before-help-to-reference-1600.png` (Help "FrogScript reference").
- After: `/tmp/claude-0/wiki/after-reference-1280.png` and `/tmp/claude-0/wiki/after-reference-1600.png` (Console), `/tmp/claude-0/wiki/after-help-to-reference-1280.png` and `/tmp/claude-0/wiki/after-help-to-reference-1600.png` (Help).
- Criterion 1: the editor text, the reference search and the output pane are visible at both sizes in the "after" shots.
- Criterion 2: the six category headings are reached by scrolling the reference (`/tmp/claude-0/wiki/after-cats-c-1600.png`, `after-cats-b-1600.png`), the topics show (`after-reference-scrolled-1600.png`), and the cookbook buttons work: "Insert into editor" at 1600 (`after-insert-1600.png`) and 1280 (`after-insert-1280.png`), and "Run in console" at 1600 opened a new tab and printed the recipe's output (`after-run-1600.png`).

Open, not part of this bug: with two tabs open and the reference open at 1280 wide, the tab row overlaps the "Hide reference" and "Save" buttons (`/tmp/claude-0/wiki/after-help-to-reference-1280-settled.png`). The tab bar is one row that does not wrap.

