# A calmer UI (proposal for the owner's acceptance)

Status: **first part built** on 2026-11-03 (TASK-0192): the owner said the library, the dashboard, the rehearsal setup, the settings and help are affected most, then the whole application. Checked with screenshots of the running app (`studyfrog --screenshot FILE --screen NAME`). The rest is still a proposal.

The owner's observation after trying 0.3.0: the UI is convoluted and cramped, and one searches for information while
being flooded with it. The five screenshots of 2026-11-03 (stack page, Create dialog narrow and wide, Home) are the
reference.

## Done in 0.3.1 (BUG-0002 to BUG-0004, TASK-0188, TASK-0189)
- Every screen has a 20 px side and 14 px top margin instead of 8 px.
- The four Home figures are 40 px apart.
- The Create dialog's header is two rows: the window controls on one, the stack and type on the other.
- The status bar's Help button is gone (the `?`, the sidebar entry and `F1` remain).

## Also built in the first part
- A reading column (`kit::column`, 780 to 860 points, centred) for the rehearsal setup, Settings and help pages, so lines are not as long as the window; section captions and grouped cards (`kit::caption`, `kit::group`); section headings are 17 points instead of 24.
- Rehearsal setup: the Back and Next buttons stay in view; stacks are one card, indented by depth, with counts in grey; the filters are a second card.
- Library: the stack list has a fixed range of width and cuts long names; the search is one row (mode, field, type, status and priority with their names inside); the table uses the height there is.
- Stack page: Start rehearsal is the one primary button, with New item and a "More actions" menu (Details, Import, Import or export); the Summary has four equal cards and two columns of grouped cards.
- Aligned rows: names and bars share two columns (`kit::bar_rows`: priorities, ratings, familiarity spread) so bars start at one place and have one length; the due rows on Home are a three-column grid.
- StudyFrog's own title bar (TASK-0190).

## Where the screens show too much at once (the proposal; marked ✔ where built)
| Screen | What is crowded | Proposal |
|---|---|---|
| Stack page header ✔ | Six buttons in one row | One primary button (Start rehearsal) and New item; the rest in "More actions" |
| Stack page tabs | Nine tabs | Keep Summary, List, Board, Calendar; the others under More at every width |
| Stack Summary ✔ (partly) | The ring, a text list of the same numbers, priorities and types in a row | Two columns of grouped cards; the ring's legend stays as text |
| Library | Stack list, sidebar and page are three columns | The stack list folds away like the sidebar and remembers it |
| Home ✔ (partly) | Three tall empty cards when there is no data | Cards capped in height; the due rows aligned; one line of guidance when empty is still to do |
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
