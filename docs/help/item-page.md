# Item page and editor

Every stack, note, card and question opens on its own item page. There you read it, edit it and see how well you
know it. Open an item by clicking its title anywhere: in the [[library]], on [[home|Home]], in the search results
of the top bar, in the [[graph]], or through a `[[wikilink]]`.

## The top row

- **Back** returns to where you came from.
- The file's path in the vault.
- Next to it, StudyFrog says **unsaved changes**, **saving…** or **Not saved** with the reason.
- **Revert** throws away your edits. **Reload from disk** reads the file again. There is no Save button.

StudyFrog saves for you: when you finish a word (you type a space or a line break), when you change something other
than text, and at the latest five seconds after your first unsaved edit. Pausing alone does not save. Leaving the
item, opening another one or exiting saves first, and a small **Saved** note appears when you leave an item you
changed. If a save fails, StudyFrog says why and tries again.

Saving is safe. If the file was changed outside StudyFrog since you opened it (for example in another editor),
StudyFrog does not overwrite it and tells you. Choose **Reload from disk** (your edits are dropped) or **Keep mine**
(your edits replace the other change).

## The text

The **Title** is at the top. Below it is a Markdown editor for every text of the item:

- a note has a body,
- a card has a **Front** and a **Back**,
- a question has a **Prompt** and a **Reference answer**,
- a stack has a description.

Each editor has a toolbar and three views: **Write**, **Split** (text and preview side by side) and **Preview**.
The toolbar has **Bold**, **Italic**, **Underline**, **Strikethrough**, **Colour**, **Code**, **Heading**, **Quote**,
**Bullet list**, **Numbered list**, **Link**, **Link to a note**, **Table**, **Rule** and **Picture**.
In a list, **Enter** starts the next item (the next number, the same bullet, or an unchecked box for a task) and **Enter** or **Backspace** on an empty item ends the list.

- **Table** inserts a small table (`| a | b |` rows under a header row). Put `:---`, `:---:` or `---:` in the second row
  to align a column left, centre or right. Cells can hold bold, italic and links.
- **Underline** and **Colour** are not Markdown, so the file keeps them as small HTML that other readers still
  understand: `<u>text</u>` and `<span style="color:#c0392b">text</span>`. The colours are red, orange, green, blue,
  purple and grey; each is shown a little darker on the light theme and lighter on the dark one. Any other HTML
  stays visible as typed.
- A line with only `[TOC]` becomes a table of contents: the headings of the note, indented by level, that you can
  click to jump to. The file stores only the marker, never a copy of the headings.
- **Code blocks** (three backticks, then a language) are coloured for python, r, rust, php, java, cpp (also c), toml,
  json, html, csv and tsv: keywords, strings, comments and numbers. Comments are also italic and the code itself is never
  changed, so nothing depends on colour. A fence without a language, or with another one, is drawn plain.
- **Macros** save typing. Type one in the text; it turns into Markdown the moment you type the closing brace (a pasted
  macro is left alone, and a name StudyFrog does not know stays as typed). The `{ }` button in the toolbar lists them:
  `{table}` or `{table:3x4}` (columns by rows), `{toc}`, `{code}` or `{code:python}`, `{quote}`, `{image}` (opens the
  picture chooser) and `{color:red}` (red, orange, green, blue, purple or grey).
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
