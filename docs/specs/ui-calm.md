# A calmer UI (proposal for the owner's acceptance)

Status: **proposed** on 2026-11-03 (TASK-0192). Nothing below except the first section is built.

The owner's observation after trying 0.3.0: the UI is convoluted and cramped, and one searches for information while
being flooded with it. The five screenshots of 2026-11-03 (stack page, Create dialog narrow and wide, Home) are the
reference.

## Done in 0.3.1 (BUG-0002 to BUG-0004, TASK-0188, TASK-0189)
- Every screen has a 20 px side and 14 px top margin instead of 8 px.
- The four Home figures are 40 px apart.
- The Create dialog's header is two rows: the window controls on one, the stack and type on the other.
- The status bar's Help button is gone (the `?`, the sidebar entry and `F1` remain).

## Where the screens show too much at once
| Screen | What is crowded | Proposal |
|---|---|---|
| Stack page header | Six buttons in one row (status, Details, New item, Import…, Import or export…, Start rehearsal) | One primary button (Start rehearsal) and New item; the rest, including one merged "Import or export…", in a "…" menu |
| Stack page tabs | Nine tabs | Keep Summary, List, Board, Calendar; the others under More at every width |
| Stack Summary | The ring, a text list of the same numbers, priorities and types in a row | The ring with its legend only; priorities and types as one compact bar each |
| Library | Stack list, sidebar and page are three columns | The stack list folds away like the sidebar and remembers it |
| Home | Three tall empty cards when there is no data | Cards as tall as their content; one line of guidance when empty |
| Create dialog | Priority, difficulty and tags always shown | Tags, priority and difficulty under "More fields" for a card; shown for a note and a question |
| Sidebar | Eleven entries | Group: Study (Home, Library, Rehearse), Plan (Calendar, Board), Insight (Reports, Achievements, Graph), then Assistant, Settings, Help |
| Type | One size for labels and values, weak captions in grey | A scale (13 / 15 / 18 / 26), captions at 13 and a stronger contrast for weak text |

## Rules for new screens
1. At most one primary button per area.
2. Group related controls; separate groups with 16 px, items inside a group with 8 px.
3. Hide what is rarely used behind a menu; do not remove it.
4. Empty states are one sentence and one action.

## Open
- Which screens bother the owner most (order of work)?
- Is the sidebar grouping wanted?
- Should the stack list fold by default?
