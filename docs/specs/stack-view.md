# Stack view (stack overview)

Planned for 0.3.0 (STORY-0063, STORY-0064). This is Jira's *project view*: the page of a stack as a place to work in, with tabs.
The stack's own details (description, priority, subjects, scheduler, due date) stay on the item page, which already serves a note, a
card, a question and a stack like Jira's issue view; the owner accepted it as it is on 2026-11-02. It is reached from the header's
"Details" button, so the stack view has no Details tab. **Status: accepted by the owner on 2026-11-02 (TASK-0149).**

## Header
- **Breadcrumb** of the parent stacks (each a link), then the stack's name as a large title.
- **Status chip** (the five statuses of [status](status.md)) that opens a menu to change it, and the roll-up of its items beside it.
- **Actions:** Details (opens the item page), New item (the Create dialog with this stack chosen), Import (the dialog of TASK-0134),
  Start rehearsal (the run setup with this stack ticked).

## Tab bar
Built on the tab widget of STORY-0060 (Left/Right keys, one tab at a time, the tab remembered **per stack** in the app settings).
Tabs that do not fit go into a **More** menu. Order: **Summary**, List, Backlog, Roadmap, Board, Calendar, Reports, Sub-stacks, Goals.
The sub-views are specified with their tasks (STORY-0064). "Forms" is not built: StudyFrog has one user.
A sub-stack is reachable both as a section of Summary and as a tab of its own page; opening a sub-stack opens its own stack view.

## Summary
Nothing is read until the tab opens.
1. **Four cards**, each a number and a label with its period in text: *completed* (items that became done, last 7 days), *updated* (items
   edited, last 7 days), *created* (items created, last 7 days), *due soon* (items due in the next 7 days).
2. **Status overview:** a ring chart with the total of items in the middle and a legend of the five statuses with count and percent.
   Colour is never the only cue: every slice is named in the legend with its number.
3. **Priorities:** a bar chart, one bar per priority, each with its count as text.
4. **Types:** a table of note, card and question with their counts.
5. **Activity:** a feed of recent changes to the stack's items (created, edited, rehearsed; a change of status is not recorded with a time, so it is not listed), newest first, 20 at most;
   empty state "No activity yet".
6. **Sub-stacks:** a list with status, progress (done / total) and familiarity of each, folding open and closed.

## Not in 0.3.0
Editing the layout, per-user card choices, watchers and automation.

## Decided by the owner
1. Backlog comes before Roadmap in the tab order.
2. Start rehearsal is in the header.
3. "Completed" counts items whose status became done in the last 7 days (the file does not record when, so the count is the items that are done and were edited in that time).
