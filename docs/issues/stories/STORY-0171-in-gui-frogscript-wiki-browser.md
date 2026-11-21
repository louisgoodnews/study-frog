---
id: STORY-0171
type: story
title: "In-App FrogScript Wiki and Standard Library Reference Browser"
status: review
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

## Open Questions
None.
