# Import and export

Export writes your library, or one stack, to a file in a documented format. Import reads a file into your vault, with
a preview first. This is for moving content in and out; to save everything, use a backup instead (see
[[settings]]).

Open the **Import and export** dialog from:

- **Import or export…** on a [[stack-page|stack page]] (the stack is chosen for you),
- **Import or export…** in the **More** menu of the [[create-dialog|Create dialog]],
- **Export…** or **Import…** in [[settings|Settings]], **Data** tab.

The dialog has two tabs: **Export** and **Import**. **Cancel** closes it.

## Export

1. **Export what**: the **Whole library** or one stack (with the stacks inside it).
2. Press **Export…** and choose the **Export mode**:
   - **Default: without scheduling** writes your items only.
   - **Advanced** also writes the review history and the schedule (JSON only).
3. Choose the format:
   - **JSON with pictures (zip)**: everything, with pictures and PDFs, in a zip. The best choice to move content to
     another StudyFrog vault.
   - **CSV (cards and questions)**: one row per card or question (stack, kind, title, front, back, tags). Opens in a
     spreadsheet.
   - **Markdown files (zip)**: your vault's own files and folders, with pictures and PDFs.
4. **Write to folder**: type a folder or press **Choose folder…**, then **Export**.

## Import

1. **File or folder to import**: type the path, or press **Choose file…** or **Choose folder…**. StudyFrog reads:
   - its own exports (JSON or zip),
   - CSV or tab-separated cards,
   - Anki packages (`.apkg`),
   - Obsidian vaults (a folder or a zip).
2. **Import into**: **As the file says**, or one stack.
3. Press **Preview**. Nothing is written yet. The preview says how many items were found, how many will be created,
   how many already exist and how many have errors, how many new stacks and picture or PDF files, any warnings, and
   the verdict for each item.
4. **Items that exist**: **Skip** them, **Replace** them (the existing file is overwritten, keeping its place), or
   **Keep both** (the new one gets a new identity).
5. For an advanced export or an Anki package, tick **Import the review history** to bring your reviews along; the
   schedule is rebuilt from them.
6. Press **Import**.

An item already exists when an item with the same identity is in the vault, or a card or question with the same front
and back is in the same stack. A cancel writes nothing.

What each format has to look like, with examples, is on [[import-formats]]. For CSV files the preview lets you choose the delimiter, the encoding and which column is which, and lists every problem at once.

## From Anki

Each deck becomes a stack. Basic and cloze notes become cards. Tags and pictures are kept. The review history can be
imported, with the schedule rebuilt.

## From Obsidian

Each folder becomes a stack and each Markdown file a note, with its front matter, wikilinks, tags and pictures.
Obsidian features StudyFrog does not support (callouts, dataview blocks, comments, highlights, block references and
note embeds) stay as plain text, and the preview counts them so you know where to look.

## Card types and importers from extensions

An extension can add an importer or a card type. When one is installed, a stack page shows **Import…** with **From a
file** and **From a card type** (for example a cloze card). Imported cards get a board card each (the first 25, then
one summary card); see [[board]].

## Limits and safety

A file of at most 64 MB, 20 000 items and a zip of at most 5 000 files. A file or folder name that would leave the
vault is refused. A broken file gives an error in the preview, never a crash.

## Related pages

[[privacy]], [[library]], [[faq]].

## Suggestions from this computer
When the Import tab opens, StudyFrog looks in the usual places and lists what it found under **Found on this computer**:

- **Obsidian vaults** from Obsidian's own list of vaults, with the number of Markdown files.
- **Anki packages** (`.apkg` files) in your Downloads, Documents and Desktop folders.
- **Anki profiles**. StudyFrog does not read Anki's own database (it is locked while Anki runs and its format changes);
  export a deck from Anki as a package, with "Support older Anki versions" ticked, and it will show up as a package.

Nothing is read except names and counts until you press **Use** next to an entry, and then you see the preview first.
