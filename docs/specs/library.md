# Library

Planned for 0.3.0 (STORY-0057). The layout (a copy of Jira's list view), the query syntax, the column defaults and saved filters per vault were all accepted by the
owner on 2026-11-02.

## Today
The Library screen puts everything in one scrolling page: a tree of stacks on the left; on the right the stack's name, its due date,
a title filter, its items, its PDFs, the new-card form and the import form. Items are found one stack at a time, nothing can be
sorted or changed in bulk, and the forms are always in the way.

## The new screen
```
+------------------+---------------------------------------------------------------+
| Stacks           | [ Standard | Query ] [ search or query ...        ] [Filters v]  |
|  v French        |  + Create   Import / export v            3 selected: Status v ... |
|    Verbs         +---------------------------------------------------------------+
|    Nouns         | [ ] Title            Type  Status       Priority  Due    ...    |
|  v Maths         | [ ] Bonjour          card  Learning     Medium    2026-11-18     |
|  Inbox           | [x] Au revoir        card  Not studied  Medium    -              |
|  All items       | ...                                                            |
|                  |                                        50 of 1000+ items  (r)  |
+------------------+---------------------------------------------------------------+
```
- **Left:** a collapsible tree of stacks, "Inbox" and "All items". Choosing a node filters the list to it and below it; the tree can be
  folded away (the same toggle as the sidebar).
- **Top:** the search bar with the Standard and Query modes ([query](query.md)), a Filters menu with the saved filters, Create (the Create dialog) and
  an Import and export menu (STORY-0065).
- **Table:** one row per item. The first column is a check box; the title is a link that opens the item page; every other column is data.
- **Bottom:** "N of M items" and a refresh button, as in Jira ("50 von 1000+"); the count says "1000+" when the list is cut at 1000 rows.

## Columns
(In the first version the direction of a sort is shown as `^` and `v` after the heading, because the app's font has no arrow glyphs.)
Shown by default: title, type, status, priority, due. Optional (chosen from a column menu, remembered per profile): difficulty,
familiarity (shown as a percentage), stack, subjects, tags, reviews, created, updated, path. A column is sorted by clicking its header (click
again to reverse, shift-click to add a second key); the sort is also what an `ORDER BY` in a query sets.

## Rows and selection
Click a title to open the item; click a row's check box, or press Space on a focused row, to select it; the header check box selects every row
that matches the filter (not only the visible ones, and the bar says how many). Up and Down move the focus, Enter opens, Ctrl+A selects all.
With a selection a bar appears with **bulk actions**: set status, set priority, set difficulty, move to a stack, add or remove a tag, delete
(after a confirmation that names the number). Every bulk action is one undoable step and writes the item files through the same code as the item page.

## Filters
- **Standard mode:** filter fields for text, type, status, stack (with its sub-stacks), subject, tag, priority, difficulty, due date and
  familiarity. The fields compose into a query; the Query mode shows it.
- **Query mode:** the typed language of [query](query.md); a query Standard cannot show is marked "Query only".
- **Saved filters:** a filter has a name and a query; they are kept in `filters.toml` in the vault (plain text, so they are backed up and
  travel with the vault) and listed in the Filters menu. (TASK-0133 said "per profile"; a query names the vault's stacks and subjects, so
  per vault is proposed.)

## From the page that is there now
| What | Where it goes |
|---|---|
| New-card form, new note, new question | the Create dialog (already there) |
| Import form | the Import and export menu |
| A stack's due date, description and sub-stacks | the stack page (STORY-0063) and the item page; the tree shows sub-stacks |
| A stack's PDFs | the stack page's PDFs tab; PDFs are not rows in the list in 0.3.0 (until the stack overview exists they stay under the list) |
| Title filter | the search bar |

## What each task takes, today and after
| Task | Today | After |
|---|---|---|
| Find a card by words | open its stack, type in "Search titles" (or use the global search, which leaves the page) | type in the search bar; the whole library is searched |
| See what is due | the home page or a run | `due <= today` in the search bar, or the Filters menu |
| Mark ten cards Complete | open each, choose, save: about 40 steps | select ten, Status, Complete: 12 steps |
| Create a card | scroll to the form under the list | Create, Card |
| Review what is hard | not possible | `familiarity < 0.4 ORDER BY familiarity` |

## Performance and accessibility
The table is virtualised: only the rows in view are drawn; 10 000 items scroll within the frame budget and a filter over 10 000 items answers in
under 50 ms (both measured in TASK-0130 and TASK-0131). Every control has a name (the accessibility audit covers the screen); colour is never the
only cue for status or priority. An empty list says why ("No items match. Clear the filter?").

## Not in this specification
Editing a cell in the table; grouping rows; PDFs as rows; a board view of the list (the stack's Board tab does that).
