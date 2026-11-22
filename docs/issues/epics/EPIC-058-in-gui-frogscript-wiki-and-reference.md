---
id: EPIC-058
type: epic
title: "In-GUI FrogScript Wiki and Interactive Documentation"
status: review
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
- [x] Integrated in-GUI FrogScript Wiki browser accessible directly within ScriptStudio and Help
- [ ] Categorized documentation covering syntax, standard library functions, and application lifecycle event hooks
- [ ] Interactive runnable code cookbook with 1-click "Try in Console" and "Insert into Editor" capabilities

## Implementation (2026-11-22)
Criterion 1 is met. The Help screen's page tree has a "FrogScript reference" button (with a vault open; without one it says why). It opens ScriptStudio with the reference panel shown and expanded, the same panel and state as the Console's "Reference" button (`WikiMsg::Show` in `studyfrog-ui/src/script_studio/wiki_panel.rs`, `studyfrog-ui/src/view/help.rs`). Test: `studyfrog-ui/tests/help_wiki.rs`. Running app: `/tmp/claude-0/epic58/02-help-1280.png` (the button in the Help tree) and `/tmp/claude-0/epic58/03-help-to-reference.png` (ScriptStudio with the panel open).

Criteria 2 and 3 are not ticked. The data is in the code and tested (`studyfrog-script` wiki and cookbook, the lifecycle-event drift test), but in the running app the panel does not show its topics or cookbook buttons: the reference beside the editor is squeezed, the topic list is blank, and the output pane goes out of the window at 1280x800 and 1600x900 (`/tmp/claude-0/epic58/05-console-reference-button.png`, `06-console-reference-1600.png`, `07-console-reference-settled.png`). The same happens from the Console's own "Reference" button, so it predates the Help link. Filed as BUG-0020. The categories, the Lifecycle Events entry and the "Insert into editor" and "Run in console" buttons have not been seen in a window.

## Design Notes
Store reference topics in clean static data structures or embedded markdown resources within `crates/studyfrog-script` and render via `studyfrog-ui`'s markdown widget. Adhere to Invariants P1 and P6.

## Dependencies
None.

## Open Questions
None.
