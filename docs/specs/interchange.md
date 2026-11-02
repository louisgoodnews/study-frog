# Interchange: export and import

Planned for 0.3.0 (STORY-0065, STORY-0067). This is interchange, not backup: [backup](../adr/) restores a whole vault; export writes a chosen part
in a documented format and import reads one, with a preview first.

## Scope and modes
Scope is the whole library or one stack (with its sub-stacks). **Export mode:** *Default* writes the items without scheduling; *Advanced* has one
option for now, **include scheduling** (the review history and the schedule). Starting an export asks for the mode.

## JSON (`format: "studyfrog-export"`, `version: 1`)
One object: `format`, `version`, `exported` (RFC 3339), `scheduling` (bool), `stacks`, `items`.
- stack: `folder` (the stack's vault-relative folder, `/`-separated), `title`, `description`, `status?`, `priority?`.
- item: `id` (26-character id), `kind` (`note`, `card`, `question`), `title`, `stack` (a stack folder), `text` (note body, card front, question prompt),
  `answer` (card back or question reference; empty for a note), `tags`, `status?`, `priority?`, `difficulty?`, `created?`, `updated?`, and
  only with `scheduling`: `reviews` (list of `{at, rating, elapsed_days}`; `at` in Unix milliseconds, `rating` one of `again`, `hard`, `good`, `easy`).
- A reader ignores unknown keys, refuses a `version` it does not know, and treats a missing optional key as absent.
- Pictures and PDFs travel in a **zip** beside the JSON: `export.json` and `files/<vault-relative path>`.

## CSV and TSV
Header `stack,kind,title,front,back,tags` (tags separated by `;`). Cards and questions only. RFC 4180 quoting; a tab-separated file is read the same way.

## Markdown
The vault layout: one folder per stack with `_stack.md`, one file per item, with the front matter, in a zip with the pictures and PDFs.

## Import: preview, duplicates, writing
1. A **dry run** reads the file into a bundle and reports what would be **created**, what **already exists** and what **fails**; nothing is written.
2. An item already exists when an item with the same id is in the vault, else when a card or question with the same front and back is in the same stack.
3. For duplicates the user chooses **skip**, **replace** (the existing file is overwritten, keeping its path) or **keep both** (the new item gets a new id).
4. A cancel writes nothing. Items are written with the front matter of the vault format; the reviews of an advanced export are written only when
   the user ticks "import review history", and the schedule is rebuilt from them.
5. Limits: a file of at most 64 MB, 20 000 items, 200 levels of stacks, a zip of at most 5 000 files; a stack folder or file name that would leave the
   vault (`..`, an absolute path) is refused. A malformed file is an error in the preview, never a crash.

## Format adapters
Importers are records in the extension registry ([EXTENDING](../EXTENDING.md)): a name, the file extensions it reads and a function from bytes to a
bundle (stacks, items, files, warnings). The first two adapters are **Anki** (`.apkg`: decks to stacks, basic and cloze notes to cards, tags and
pictures kept, review history imported or left out) and **Obsidian** (a vault folder: folders to stacks, Markdown files to notes with front matter,
wikilinks, tags and pictures kept; syntax StudyFrog does not support stays as text and is counted in the preview). More formats are added the same way.
