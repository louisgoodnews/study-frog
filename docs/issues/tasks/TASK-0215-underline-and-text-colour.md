---
id: TASK-0215
type: task
title: "Underline and text colour"
status: done
epic: EPIC-026
parents: [STORY-0074]
estimate: 3
labels: [m11, ui, md]
acceptance_criteria:
  - "Underline and a text colour (from a small palette that works in both themes) can be applied from the toolbar and with {color:NAME}"
  - "The preview draws them, and the file keeps them as the HTML of TASK-0197"
  - "Colours are never the only carrier of meaning in the app's own screens"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Two of the owner's formatting wishes that Markdown has no syntax for.

## Acceptance Criteria
- [x] Underline and a text colour (from a small palette that works in both themes) can be applied from the toolbar and with {color:NAME}
- [x] The preview draws them, and the file keeps them as the HTML of TASK-0197
- [x] Colours are never the only carrier of meaning in the app's own screens

## Design Notes
Done as `hue.rs` (the six names and values) and `theme_hue.rs` (a shade per theme, the text colour under high contrast), tested for contrast on every background. {color:NAME} comes with the macros (TASK-0198).

Palette as plain data with names (red, orange, green, blue, purple, grey) and a value per theme. The span's style is parsed with a whitelist; anything else is shown as typed.

## Dependencies
TASK-0197.

## Open Questions
None.
