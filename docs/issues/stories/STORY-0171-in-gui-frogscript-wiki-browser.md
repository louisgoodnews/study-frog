---
id: STORY-0171
type: story
title: "In-App FrogScript Wiki and Standard Library Reference Browser"
status: backlog
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
- [ ] Documentation side-panel or modal tab in ScriptStudio displaying categorized topics
- [ ] Topic coverage includes Syntax, Operators, Control Flow, Built-in Functions, and Vault Objects
- [ ] Search filter allowing rapid lookup of functions by keyword or return type

## Design Notes
Implement lightweight searchable tree navigation in egui, rendering selected topic documentation via the existing markdown viewer.

## Dependencies
EPIC-058

## Open Questions
None.
