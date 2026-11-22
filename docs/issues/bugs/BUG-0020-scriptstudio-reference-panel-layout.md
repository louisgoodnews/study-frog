---
id: BUG-0020
type: bug
title: "ScriptStudio reference panel is squeezed and its topics are not shown"
status: backlog
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
- [ ] With the FrogScript reference open beside the ScriptStudio editor, the editor text, the reference search and the output pane are all visible at 1280x800 and at 1600x900
- [ ] The reference lists every category heading and its topics, and the cookbook's Insert into editor and Run in console buttons are visible and clickable

## Design Notes
- Touchpoints: `crates/studyfrog-ui/src/view/console.rs` (`studio_editor`, the editor and reference columns), `crates/studyfrog-ui/src/view/studio_reference.rs`, `crates/studyfrog-ui/src/view/console/code.rs`.
- Guardrails: P4 (nesting < 3) and P6 (files <= 250 lines).

## Dependencies
- EPIC-058

## Open Questions
- None.
