---
id: STORY-0171
type: story
title: "In-App FrogScript Wiki and Standard Library Reference Browser"
status: done
epic: EPIC-058
parents: ['EPIC-058']
estimate: 5
labels: [m14, ext, ui]
acceptance_criteria:
  - 'Documentation side-panel or modal tab in ScriptStudio displaying categorized topics'
  - 'Topic coverage includes Syntax, Operators, Control Flow, Built-in Functions, and Vault Objects'
  - 'Search filter allowing rapid lookup of functions by keyword or return type'
---

## Context
Writing FrogScript requires instant access to language syntax and standard library signatures without leaving the ScriptStudio editor.

## Description
Develop the in-GUI Wiki browser:
- Provide a "Wiki & Docs" tab or slide-out drawer inside ScriptStudio.
- Structure content into intuitive categories: Language Basics, Control Flow, Collections, Vault Access, and Lifecycle Events.
- Provide a fast search filter box to instantly locate methods, signatures, and descriptions.

## Acceptance Criteria
- [x] Documentation side-panel or modal tab in ScriptStudio displaying categorized topics
- [x] Topic coverage includes Syntax, Operators, Control Flow, Built-in Functions, and Vault Objects
- [x] Search filter allowing rapid lookup of functions by keyword or return type

## Design Notes
Implement lightweight searchable tree navigation in egui, rendering selected topic documentation via the existing markdown viewer.

## Dependencies
EPIC-058

## Implementation (2026-11-21)
The reference is data in `studyfrog-script::wiki_topics` (name, category, signature, return type, summary, example), searched by `wiki::search` (name, signature, return type, summary and category, without case). Its categories are the ones the criteria name: Syntax, Operators, Control Flow, Built-in Functions and Vault Objects.

It is checked against the language itself: `tests/wiki.rs` reads `eval_stdlib.rs` and `token.rs`, so a new keyword or built-in without an entry fails. The first version of the drift test found three gaps (`stack`, `type` and the function `tag`), which are now filled.

In ScriptStudio the panel opens beside the editor ("Reference" in the tab bar). Its "Add example to the script" button appends the example to the active tab. The panel shows the language's behaviour as the code has it: `today` is a day number, while the spec describes a date type that is not built yet.

Tests: `studyfrog-script/tests/wiki.rs`, `studyfrog-ui/tests/studio_reference.rs`. The script editor now has an accessible name ("Script text"), which the console tests check.

## Implementation (2026-11-22)
The description also names a Lifecycle Events category, which the first pass did not have. It is now there: `wiki_topics_events` lists one entry for each event kind of the bus (`studyfrog-ui::script_events::EVENT_KINDS`, fourteen in all), each with the `on = [...]` line that starts a job on it. The category is the sixth heading in the panel. The drift test `every_lifecycle_event_kind_has_an_entry_in_its_category` in `studyfrog-ui/tests/studio_reference.rs` fails when an event has no entry or an entry names no event. The reference's `CATEGORIES` test in `studyfrog-script/tests/wiki.rs` now expects six headings.

Not done: the epic's "Help" (F1) entry point is not built. This story's criteria are about ScriptStudio, so it is left for EPIC-058.

Needs a person at a window: the look of the new category heading and the panel's layout with six categories, in light and dark themes.

Review (2026-11-22): criteria 1 to 3 NOT MET in the running app. The panel opens with its heading and search box (`/tmp/claude-0/review-c/shots/13-reference.png`), but no categorized topics, topic coverage or search results are visible at 1280x800 or 1600x900 (`13-reference.png`, `/tmp/claude-0/review-c/shots/14-reference-1600.png`, `/tmp/claude-0/review-c/shots/15-reference-search.png`). Blocked by BUG-0020. Status stays review.

## Window check (2026-11-22, EPIC-058)
The panel opens from ScriptStudio's "Reference" button and from the Help screen's "FrogScript reference" button, and its heading and search box show in the window. The topics do not: beside the editor the list is blank, even after scrolling, and the editor text is squeezed, at 1280x800 and at 1600x900 (BUG-0020). The three criteria above name what the panel shows, so they are unticked here. The data and the search are covered by `studyfrog-script/tests/wiki.rs` and `studyfrog-ui/tests/studio_reference.rs`, but that is not a window check. Screenshots: `/tmp/claude-0/epic58/03-help-to-reference.png`, `05-console-reference-button.png`, `07-console-reference-settled.png`.

## Verification (2026-11-22, after BUG-0020)
The reference now shows in the running app beside the editor, at 1280x800 and 1600x900, from the Console's "Reference" button and the Help screen's "FrogScript reference" button (`/tmp/claude-0/wiki/after-reference-1280.png`, `after-reference-1600.png`, `after-help-to-reference-1280.png`, `after-help-to-reference-1600.png`).
- Criterion 1 (side-panel with categorized topics): the panel lists the topics under their category headings (`after-reference-1600.png`, `after-reference-scrolled-1600.png`).
- Criterion 2 (coverage): the headings Syntax, Operators, Control Flow, Built-in Functions, Vault Objects and Lifecycle Events were each seen in the running window at 1600x900 by scrolling (`after-cats-c-1600.png`, `after-cats-b-1600.png`, `after-reference-scrolled-1600.png`, `after-search-1600.png`). The test `studio_reference_layout.rs` checks each category heading and topic at 960x640, 1280x800 and 1600x900.
- Criterion 3 (search): typing "len" in the search field showed "Built-in Functions: len" and "Vault Objects: calendar, run" and hid the rest (`after-search-1600.png`).
Tests: `studyfrog-ui/tests/studio_reference_layout.rs`, `studio_reference.rs`, `help_wiki.rs`. Still open for a person at a window: the look in the light theme, which was not checked.

## Open Questions
None.
