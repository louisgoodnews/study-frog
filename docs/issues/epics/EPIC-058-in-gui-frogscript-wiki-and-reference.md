---
id: EPIC-058
type: epic
title: "In-GUI FrogScript Wiki and Interactive Documentation"
status: backlog
parents: []
labels: [m14, ext, ui]
acceptance_criteria:
  - 'Integrated in-GUI FrogScript Wiki browser accessible directly within ScriptStudio and Help'
  - 'Categorized documentation covering syntax, standard library functions, and application lifecycle event hooks'
  - 'Interactive runnable code cookbook with 1-click "Try in Console" and "Insert into Editor" capabilities'
---

## Context
FrogScript and ScriptStudio provide extensive automation capabilities, but without in-app documentation or a Wiki, beginners and non-programmers find it impossible to write or understand scripts without hunting through external codebase markdown files.

## Description
Embed an interactive, browsable FrogScript Wiki directly within the StudyFrog GUI:
1. **Interactive In-GUI Wiki**: A searchable documentation browser integrated into ScriptStudio and the Help center (`F1`), detailing syntax, control flow, built-in functions, and data types.
2. **API & Event Hook Reference**: Full API specifications for vault manipulation, item creation, review querying, and event bus hooks (`vault_opened`, `rehearsal_finished`, etc.).
3. **Runnable Cookbook Recipes**: A library of practical script examples (e.g., auto-tagging overdue cards, weekly review digest generation, custom export formatting) with 1-click actions to run them in the console or copy them into the editor.

## Acceptance Criteria
- [ ] Integrated in-GUI FrogScript Wiki browser accessible directly within ScriptStudio and Help
- [ ] Categorized documentation covering syntax, standard library functions, and application lifecycle event hooks
- [ ] Interactive runnable code cookbook with 1-click "Try in Console" and "Insert into Editor" capabilities

## Design Notes
Store reference topics in clean static data structures or embedded markdown resources within `crates/studyfrog-script` and render via `studyfrog-ui`'s markdown widget. Adhere to Invariants P1 and P6.

## Dependencies
None.

## Open Questions
None.
