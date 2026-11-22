---
id: STORY-0172
type: story
title: "Interactive FrogScript Cookbook and 1-Click Code Templates"
status: review
epic: EPIC-058
parents: ['EPIC-058']
estimate: 3
labels: [m14, ext, ui]
acceptance_criteria:
  - 'Curated cookbook of at least 8 ready-to-run script recipes for common study automation tasks'
  - '1-click "Insert into Editor" button copying cookbook code directly into the active editor tab'
  - '1-click "Run in Console" button executing sample scripts with instant output preview'
---

## Context
Non-programmers learn languages most effectively through concrete, runnable examples that solve immediate real-world problems.

## Description
Provide an interactive cookbook of practical scripts within the FrogScript Wiki:
- Include common automation templates: Bulk card tagger, overdue review analyzer, weekly progress reporter, vocabulary cleaner, and custom backup trigger.
- Add an "Insert into Editor" button on every recipe code block that safely copies the template into a new script tab.
- Add a "Run in Console" action that executes read-only query recipes immediately and displays the output in the console stream.

## Acceptance Criteria
- [ ] Curated cookbook of at least 8 ready-to-run script recipes for common study automation tasks
- [ ] 1-click "Insert into Editor" button copying cookbook code directly into the active editor tab
- [ ] 1-click "Run in Console" button executing sample scripts with instant output preview

## Implementation (2026-11-21)
`studyfrog-script::cookbook` holds ten recipes: seven read-only (overdue review analyzer, due today, weekly progress, stack summary, exams in the next two weeks, tag census, backup reminder) and three that change items (bulk card tagger, vocabulary cleaner, priority reset). The cookbook sits at the foot of the FrogScript reference panel.

"Insert into editor" adds a recipe to the end of the active tab, as the acceptance criteria say. The description asked for a new tab; the criteria win, so the user's work is only ever appended to. "Run in console" is offered only for read-only recipes; it opens a new tab, loads the recipe and runs it, so the user's tab is kept and the output reaches the console.

The backup recipe is a reminder: the language cannot start a backup, and the recipe says so.

Tests: `studyfrog-script/tests/cookbook.rs` (parse, run on an empty engine, no writes in read-only recipes) and `studyfrog-ui/tests/studio_reference.rs` (every recipe offered, run path, appending). Recipes that reach `vault.` or `calendar.` are not run by the engine test, because the app provides those names when it runs a script.

## Implementation check (2026-11-22)
Re-checked against the criteria: ten recipes, "Insert into editor" on every recipe and "Run in console" on the seven read-only ones. No code changed here. The tests `the_cookbook_offers_every_recipe_and_runs_the_read_only_ones` and `running_a_recipe_opens_a_new_tab_and_keeps_the_users_tab` in `studyfrog-ui/tests/studio_reference.rs` still pass.

Needs a person at a window: that the recipe buttons read well in the panel and that the console output appears where expected after "Run in console".

Review (2026-11-22): criteria 1 to 3 NOT MET as seen in the window. No recipe, "Insert into editor" or "Run in console" button is visible at 1280x800 or 1600x900 (`/tmp/claude-0/review-c/shots/13-reference.png`, `/tmp/claude-0/review-c/shots/14-reference-1600.png`). Blocked by BUG-0020. Status stays review.

## Window check (2026-11-22, EPIC-058)
The cookbook is at the foot of the reference panel, which the window does not show (BUG-0020): the recipes, "Insert into editor" and "Run in console" are not visible at 1280x800 or 1600x900, so no click was made on them. The three criteria are unticked until they are seen in a window. The ten recipes and the buttons are covered by `studyfrog-script/tests/cookbook.rs` and `studyfrog-ui/tests/studio_reference.rs`. Screenshots of the panel: `/tmp/claude-0/epic58/07-console-reference-settled.png`.

## Design Notes
Keep cookbook templates self-contained and documented with inline comments.

## Dependencies
EPIC-058

## Open Questions
None.
