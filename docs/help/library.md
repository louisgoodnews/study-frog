# Library

The library shows every item in your vault in one list. Use it to find items, sort them, and change many at once.
Open it from the sidebar.

## The tree on the left

- **Stacks**: every stack, with the stacks inside it indented. Click a stack to open its [[stack-page|stack page]].
- **Inbox**: items that are not inside any stack (only shown when there are some).
- **All items**: the whole vault in one list.
- **Overview**: every stack as a card with its counts and scheduler, and a **Problems** section that lists files
  StudyFrog could not read, with the reason.
- **New stack…** opens the Create dialog for a stack. See [[create-dialog]].

## The list

Every item is a row. The title opens the [[item-page|item page]].

- **Sort**: click a column heading to sort by it, click again to reverse, and shift-click to add a second key. The
  direction is shown after the heading as `^` or `v`.
- **Columns**: the **Columns** menu chooses which columns are shown: title, type, status, priority, difficulty,
  familiarity, stack, subjects, tags, reviews, due, created, updated and path. Your choice is remembered.
- The line above the list says how many items match, for example "50 of 1000 items".
- Only the rows on screen are drawn, so even 10 000 items scroll smoothly.

## Searching

The search bar has two modes. Switch with **Standard** and **Query** next to **Search mode**.

- **Standard**: type words of a title in **Search titles** and choose a **Type**, a **Status** and a **Priority**.
- **Query**: type a query such as `type = card AND familiarity < 40%`. Mistakes are shown under the box with the
  column where they are. While you type, field names, operators and values are suggested. See
  [[query-language|The query language]].

Switching from Standard to Query turns your fields into a query. Switching back fills the fields again when the
query is simple enough; otherwise the fields keep their old values and the app says so.

## Saved filters

The **Filters** menu keeps searches you use often.

- **Save current filter** asks for a name and saves the current fields or query.
- Choose a saved filter to use it again.
- **Remove** deletes it.

Saved filters are kept in `filters.toml` in the vault. They belong to the vault, are plain text and are part of
backups.

## Changing many items at once

Tick the check box of a row to select it. The check box in the heading selects every row the filter lets through,
not only the visible ones.

With a selection, a bar appears:

- **Set status**, **Set priority**, **Set difficulty**
- **Move to stack**
- **Add tag** and **Remove tag**
- **Delete** (asks first and names the number of items)
- **Clear selection**

Each change is one step. **Undo last change** takes it back.

## Where other things went

- To create an item, use **+ Create** in the top bar. See [[create-dialog]].
- To import or export, use a stack's page, the Create dialog's **More** menu, or Settings. See [[import-export]].
- A stack's PDFs, due date and tabs are on its [[stack-page|stack page]].

## Related pages

[[statuses|Item statuses]], [[familiarity]], [[getting-started|Getting started]], [[faq]].
