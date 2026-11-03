# Item page and editor

Every stack, note, card and question opens on its own item page. There you read it, edit it and see how well you
know it. Open an item by clicking its title anywhere: in the [[library]], on [[home|Home]], in the search results
of the top bar, in the [[graph]], or through a `[[wikilink]]`.

## The top row

- **Back** returns to where you came from.
- The file's path in the vault.
- **unsaved changes** shows while you have edits that are not saved.
- **Save** writes the file. **Revert** throws away your edits. **Reload from disk** reads the file again.

Saving is safe. If the file was changed outside StudyFrog since you opened it (for example in another editor),
StudyFrog does not overwrite it and tells you. Reload it, then make your edit again. If you try to leave with
unsaved changes, StudyFrog asks you to save or revert first.

## The text

The **Title** is at the top. Below it is a Markdown editor for every text of the item:

- a note has a body,
- a card has a **Front** and a **Back**,
- a question has a **Prompt** and a **Reference answer**,
- a stack has a description.

Each editor has a toolbar and three views: **Write**, **Split** (text and preview side by side) and **Preview**.
The toolbar has **Heading**, **Bold**, **Italic**, **Code**, **Quote**, **Bullet list**, **Numbered list**,
**Rule**, **Link**, **Link to a note** and **Picture**.

- **Picture** copies a png, jpg or gif into the vault's `assets/` folder and inserts it as
  `![alt](assets/name.png)`. Pictures show in the preview, on the item page and while rehearsing. Only pictures
  inside the vault are shown.
- **Link to a note** inserts a wikilink, `[[Title]]`. Wikilinks in the preview can be clicked.
- A bar under each editor makes it taller: drag it, or focus it and use the Up and Down arrow keys.

The preview shows headings, emphasis, lists, task items, quotes, code, wikilinks, web links and pictures. A link
like `[page 3](file.pdf#page=3)` opens the [[pdf|PDF viewer]] at that page.

## The details column

- **Kind** and the stack the item is in.
- **Status**: a menu with Automatic, Not studied yet, Learning, In review, Complete and On hold. Automatic lets
  StudyFrog work it out from your reviews. See [[statuses|Item statuses]].
- **Priority** (lowest to highest) and **Difficulty** (easy, medium, difficult). An item without its own takes the
  value of its stack.
- **Subjects**: an item takes the subjects of its stack. A stack chooses its subjects here; add new ones in
  [[settings|Settings]].
- **Tags (comma separated)**.
- **Created** and **Updated** dates.
- **Links to**: the items this one links to. **Linked from**: the items that link here.
- **Open source**: for an item made from a PDF, opens the PDF at the right page.

## Familiarity

The **Familiarity** card shows how familiar you are with the item as a percentage, what drives the value (ratings,
views, your mark, time without contact) and how it changed over time. See [[familiarity]].

## Time spent looking

While an item page is open, StudyFrog records how long you look at it: active time only, so a minute without input
stops the clock. One record is kept per visit, and one more when you change the item. This feeds familiarity. You
can switch it off in [[settings]] (Study and scheduling, **Record how long I look at items**).

## Related pages

[[create-dialog|Creating items]], [[stack-page]], [[keyboard]], [[faq]].
