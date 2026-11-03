# Board

The board is a kanban board for your study tasks: what to do, what you are doing, what is done. Open **Board** in
the sidebar.

## Columns

A new vault starts with four columns: **Backlog**, **Doing**, **Review** and **Done**. Each column shows how many
cards it holds. A column can have a **WIP limit** (work in progress): Doing allows three cards at a time. A column
over its limit is marked "Over the WIP limit".

Limiting work in progress keeps you focused: finish something before you start the next thing.

## Cards

- Type a title in **New card title** and press **Add card**. The card is added to the first column.
- Every item you make in the Create dialog, and the items of an import, also get a card in the first column (the
  first 25 of an import, then one summary card). See [[create-dialog]] and [[import-export]].
- **Edit** a card to change its **Card title**, **Card notes**, **Estimate (points)** and **Linked stack**, then
  press **Save card**. **Close card** closes the editor.
- **Delete** removes a card.

## Moving cards

- **Drag** a card by its whole face to another column. Columns light up when you can drop the card there.
- Or use the buttons on each card, which work from the keyboard and with a screen reader:
  - **Push** moves it to the top of the next column,
  - **Pull** moves it back to the top of the previous column,
  - **Move up** and **Move down** change its place in the column.

A move into a full column is refused and the reason is shown. Tick **Allow exceeding WIP limits** to move it
anyway.

## Two different boards

This board is free-form: its columns are yours, and moving a card does not change the item it is about. The
**Board** tab of a [[stack-page|stack page]] is different: it has a column per status, and moving an item there
changes its status. See [[statuses|Item statuses]].

## Where the board is kept

The board is stored in the vault's database and is part of backups. See [[privacy]].

## Related pages

[[calendar]] for dates and sessions, [[getting-started|Getting started]], [[faq]].
