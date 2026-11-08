# Changelog

All notable changes to StudyFrog are recorded here, newest first.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The project is pre-release, so
everything is under `Unreleased` until the first tagged version. Each entry names the work item
(`TASK-xxxx`) it belongs to where one exists.

## [Unreleased]

### Added
- More of the MCP server (STORY-0118): resources `studyfrog://reports/{kind}` (retention, forecast, familiarity, calibration, runs, latency) and `studyfrog://diary/{date}`; an item's resource also carries its FSRS or SM-2 schedule in every stack and finds items by key; stacks by folder or title; the summary has due and new counts, items by kind and the streak; read-only tools `vault_get_due_queue`, `analytics_get_report`, `forecast_simulate_workload` and `pdf_read_pages`. Leech diagnostics and a study-minute forecast are not built yet.
- The MCP server can also listen on local HTTP: `studyfrog --mcp --mcp-http 127.0.0.1:<port>` serves `POST /mcp` (Streamable HTTP, JSON answers) on 127.0.0.1 only, wants a bearer token (`mcp_http` under `[keys]` in `secrets.toml`, else a fresh one printed to standard error) and refuses foreign `Host` and browser `Origin` headers against DNS rebinding (STORY-0120).
- A control protocol for AI agents and scripts: `studyfrog --control-stdio [--vault PATH]` runs headless and answers one JSON line per request (words such as `nav library`, `query type = card`, `item create card "Title" --stack french`, `rehearse flip`, or JSON with an id); the window and headless instances serve `run/control.sock` in the application home (with `run/control.json`), and `studyfrog ctl <command>` sends one request to the running instance; `state` returns a snapshot of the screen, vault, item page, dialog, rehearsal and notices, `screenshot` pictures the window, and `logs --follow` streams new `console.jsonl` entries; destructive commands need `--control-allow-destructive`, `--control-read-only` refuses writes, `--no-control` keeps the window from serving the socket (STORY-0124).
- Item keys: every new stack, card, note and question gets a short key such as `STACK_1`, `FLASHCARD_12`, `NOTE_3` or `QUESTION_7` in its front matter; "Assign keys…" in the Library gives older items theirs after a confirmation (undoable); the query language has `key`, an exact key typed into the top-bar or Library search finds its item, and keys show in a Library column, the item page's toolbar (with "Copy key") and the Details tab (STORY-0096).
- Item colours: a `color` in front matter (palette word or `"#RRGGBB"`), chosen in the item page's Details > Properties (swatches, None, picker, code); it tints the item page, its Details cards, the rehearsal card and the item's Library row (with a dot), keeping text at a 4.5:1 contrast in both themes (STORY-0097, STORY-0123).
- `studyfrog --screenshot FILE --screen item:<ULID>` pictures an item's page (STORY-0123).
- Store migration 24: `item.key`, `item.color` and a `key_counter` table, so a deleted item's key is not handed out again (STORY-0096).

## [0.5.0] - 2026-11-08

### Added
- A Model Context Protocol server for AI clients: a pure JSON-RPC 2.0 router with plain-data tools, resources and prompts and a policy for writes (STORY-0117), the read-only vault summary, stack and item resources and a search tool using the query language (STORY-0118), and `studyfrog --mcp [--vault PATH]` over standard input and output with "Copy MCP server config" in the Assistant's connections (STORY-0120).
- The Reports screen has "Export as CSV": one RFC 4180 table with the review total, retention, answer times, the review forecast, the familiarity distribution and the run log, written to `exports/` in the vault (STORY-0098).
- Many more built-in achievement tiers: streaks of 3, 14, 60, 100 and 365 days, runs up to 100, ratings up to 5,000, mastery up to 500, and four tiers of learned and mastered items for every stack; the ids of the first set are unchanged, so earlier unlocks stay (STORY-0086).
- The achievements screen has a filter (all, global, one stack), groups its entries by category and stack, and a stack's page shows how many of its achievements are unlocked (STORY-0087).
- Google Gemini (`generateContent`, key in the `x-goog-api-key` header only, safety stops reported with their reason) and DeepSeek (`/chat/completions`, default address and models) as providers of the assistant; choosing a protocol fills in its first suggested model (STORY-0083, STORY-0084).
- Four more standard prompts: explain a file at three levels, be examined on a file, learn a file by Socratic dialogue, and map the concepts of a file; their worked examples are read by the importer in the tests (STORY-0088, STORY-0089, STORY-0090, STORY-0091).
- Prompts have categories (create content, tutoring, analysis), a search, and options that are put into the text, such as the depth of an explanation or the number of exam questions; Save and the assistant run use the chosen options (STORY-0092).
- The prompt library is a screen of its own, "Prompts" in the sidebar; the Assistant's Prompts tab links to it (TASK-0233).
- StudyFrog keeps its own files in one application home (`$STUDYFROG_HOME`, else `$XDG_DATA_HOME/studyfrog`, else `~/.local/share/studyfrog`): `config/`, `models/`, `logs/`. Files of the old `~/.config/studyfrog` are copied in on first start and left in place; API keys end up readable by the owner only (STORY-0122).
- Every notice is written to a rotating text log (`logs/studyfrog.log`) and a JSON Lines log (`logs/console.jsonl`), 5 MB x 3 files each (STORY-0122).
- `studyfrog --print-paths` lists the application's directories, and `studyfrog logs [--tail N] [--follow] [--json]` shows its log (STORY-0122).
- Settings > Data lists the application files with a Copy button for each path (STORY-0122).
- Benchmarks for parsing a document and for drawing a thumbnail and a page at 2x, next to the existing page benchmark (STORY-0085).

### Changed
- The assistant's call log moved from `~/.local/state/studyfrog/agent.log` to `logs/agent.log` in the application home (STORY-0122).
- The PDF viewer trims full-size page pictures (6) and thumbnails (60) apart, so scrolling the sidebar thumbnails no longer pushes out the page being read and far fewer large page textures are kept in memory; the sidebar and the arrange grid share one thumbnail scale (STORY-0085).

### Fixed
- PDF page, sidebar thumbnail and arrange-grid textures are keyed by their pixel size instead of the requested scale, so a low-resolution thumbnail shown while the sharp render is in flight no longer takes over the sharp texture and leaves the page blurred at some zoom levels or fit modes (BUG-0010).

## [0.4.3] - 2026-11-06

### Fixed
- Both CSV importers (`studyfrog-ext-example` and `studyfrog-service::xfer_csv_map`) now merge unquoted multiline continuation rows in headed CSV files before splitting columns, fixing import failures such as `line 5: a card needs at least a front and a back` when AI-generated CSV exports contain unquoted newline breaks or numbered lists inside `front` or `back` fields (BUG-0014).

### Added
- Stacks can now be moved between a parent stack and standalone (top-level) after creation via `"Move stack…"` in the stack page's `"More actions"` menu and the `"Parent stack"` selector in the stack's `Properties` tab, while preserving all items' primary stack associations and SRS schedules (STORY-0100).
- Items (notes, cards, and questions) can now be deleted directly from the item editor toolbar (`"More actions"` -> `"Delete item…"`) with an inline confirmation prompt and full `"Undo last change"` support in the library bar (STORY-0101).

## [0.4.2] - 2026-11-05

### Fixed
- Rehearsal card prompts and answers now wrap cleanly across spans (`TextWrapMode::Wrap`) and scroll vertically when taller than the viewport, and CSV rows containing unquoted commas inside `front` or `back` fields (such as LLM-generated 6-column `stack,kind,title,front,back,tags` outputs) automatically reconstruct the full question, answer, and tags instead of truncating at the first comma (BUG-0012).
- The rehearsal result (`Run finished`) screen is wrapped in an outer vertical `ScrollArea`, the `Review` section stretches dynamically with viewport height inside its own inner `ScrollArea` (instead of being capped at 140 px), and every review item card always displays `Suggested rating:` alongside reading word count and time even when no rating override is needed (BUG-0013).

## [0.4.1] - 2026-11-05

### Fixed
- The card file importer automatically detects and parses both CSV (comma-separated, RFC 4180 quoted values, and multi-column AI prompt headers `stack,kind,title,front,back,tags`) and TSV (tab-separated) formats without requiring manual conversion (BUG-0011).

### Changed
- The item page details column is restructured into a four-tab notebook layout (`Properties`, `Schedule`, `Relations`, `Learning`) with inline mutable fields and direct stack due date editing (STORY-0093).
- The Create dialog exposes priority for stacks and due date / reminder selection across all creatable item types, syncing deadlines directly to the vault calendar (STORY-0094).
- Selecting Note in the Create dialog automatically expands the modal to fill the window and defaults the Markdown editor to Write mode while retaining drag-resize, shrink toggle, and item kind switching (STORY-0095).
- The top-bar Vaults switcher popup displays clean folder basenames in borderless selectable rows with automatic ancestor disambiguation for duplicate folder names and full paths on hover (STORY-0099).

## [0.4.0] - 2026-11-04

### Fixed
- Reading two Anki packages at the same time no longer shares one temporary file, which made the Anki import tests fail now and then.
- The Board's columns share the width there is, and the new-card field leaves room for its button, so the last column and "Add card" no longer run off the right edge (TASK-0226).
- PDFs are named by their file name in the Home card, the stack page, the search results and the viewer's header; the title inside the file is added only when it says something, so files whose title is a placeholder such as "PowerPoint-Präsentation" are told apart (BUG-0009).
- Enter in the Markdown editor continues a list: the next number, the same bullet or an unchecked box for a task item, at the same indent (in the middle of an item it splits the item); Enter or Backspace on an empty item removes its marker and ends the list (BUG-0007).
- Bullets, numbers and check boxes in the preview are level with the first line of their item (BUG-0008).
- The window buttons are anchored to the window's right edge, so the red close button is no longer cut off, and there is a gap between maximise and close (BUG-0005).
- The roadmap takes the width and the height there are, even when its dates are close together, with larger rows, bars and text (BUG-0006).

### Changed
- A rating in a run writes one review and one new memory state for every selected stack that holds the item, each with that stack's own scheduler and retention; queues treat an item held by several selected stacks as one candidate, due when it is due in any and new when any of them has not scheduled it; the daily limits and Home's due counts count each stack on its own; the calendar shows and moves an item's due date per stack (TASK-0218).
- Familiarity is worked out for each (item, stack) from the reviews that stack made and its own exam, and an item's overall value is the average over its stacks: the item page lists the average familiarity and difficulty and, for each stack, whether it is the item's own or a reference, its familiarity, difficulty and due date; the library column shows the average with the per-stack values on hover; the Needs attention card, the reports, the stack percentages, the queues and the familiarity replay use the stack's own value (TASK-0219).
- Reviews and the FSRS and SM-2 states are kept for each (item, stack) (schema v21): every existing row becomes the row of the item's primary stack, or of the Inbox for a loose item, and the store reads and writes a stack's own state (`*_in` functions). The screens still show the primary stack's value until the per-stack views arrive; imported review histories are written after the reindex so they land in the item's stack (TASK-0218).
- Decided and written down: stack folders, items referenced by several stacks and an independent memory per stack ([ADR 0011](docs/adr/0011-stack-folders-and-references.md), `markdown-schema.md`, `srs-algorithm.md`; TASK-0210). The implementation follows in TASK-0211 and the tasks after it.
- The Markdown editor's code is split into a module of small files (`view/md_editor/`: toolbar, resizing, preview cache, list keys).

### Added
- Documentation, help and release notes for PDF export: in-app help page and FAQ entry explaining export routes, options, supported and unsupported elements; comprehensive showcase note demonstrating all supported elements in `examples/pdf-export/complete-note.md`; round-trip export and text extraction test in `studyfrog-service`; manual window verification and draft release notes for 0.4.0 (TASK-0232).
- Export to PDF in the application: dialog with page size, margins, font size, title page, TOC, page numbers, and card/question answers; live preview of page count and sample rendered pages before writing; entry points in item page More actions, library bulk bar, stack page More actions, and the Transfer/Export screen; notifications and toasts that open the exported PDF directly in the viewer (TASK-0231).
- Self-assessment against measured results: pure calibration report in `studyfrog-report` correlating self-rated ratings (understanding, retention, focus, mood) against objective review accuracy and item familiarity; bias analysis identifying overconfidence or underconfidence; dedicated "Self-assessment" tab in the Reports screen with metric cards, entry count indicators, and chronological calibration timeline (TASK-0174).
- Prompt for a diary entry after a rehearsal: compact self-assessment card on the rehearsal summary screen pre-filling session duration, linked run ULID, stack name, and reviewed item count; exposes 1..=5 buttons for user primary scales (energy, focus, satisfaction) with descriptive level tooltips; skippable with one key (Esc) without delaying leaving the summary (TASK-0173).
- Export use cases for PDF in the service: pure export planning and transactional execution in `studyfrog-service` for single items, selected item sets, and stacks (including sub-stacks and referenced items deduplicated exactly once); resolves internal wikilinks and vault picture attachments with labelled placeholder fallback; atomic writes via temp file and rename; hostile path refusal; cooperative cancellation and progress reporting (TASK-0230).
- Diary screen: full-featured Diary screen supporting list and month-calendar views, rich draft entry creation and editing form, 13 standard self-assessment scales with level descriptions, user-defined custom scales in Settings > Diary and reflections, tag, stack, and rating filters, date navigation, and delete confirmations (TASK-0172).
- Markdown layout and PDF writer: pure layout engine in `studyfrog-pdf` paginating Markdown AST into pages with wrapped text, syntax-highlighted code blocks, tables with aligned columns, list items, task checkboxes, and blockquotes; PDF writer built on `lopdf` producing searchable PDFs with TrueType `/ToUnicode` CMaps, document bookmarks (`/Outlines`), clickable web and internal wikilinks, and document metadata (TASK-0229).
- Folders view and Items view in the application: sidebar navigation distinguishes Folders (Obsidian-modelled file tree explorer) and Items (catalog table); Folders view includes a top toolbar (New note, New folder with stack option, Sort order, Expand/collapse all), virtualized indent tree rendering, file kind badges, context menus, drag-and-drop moves, cross-stack move confirmation dialogs, inline rename (`F2`), and keyboard navigation (TASK-0204).
- Diary entries in data model, Markdown, and index: `DiaryEntry` and rating types in `studyfrog-core`; `kind: diary` frontmatter parsing, rendering, and lossless round-tripping in `studyfrog-md`; schema migration 23 and index tables (`diary_entries`, `diary_runs`, `diary_stacks`, `diary_items`, `diary_exams`, `diary_tags`) with day-listing, text search, and item-link queries in `studyfrog-store` (TASK-0171).
- Folder operations in the service: pure plans and atomic transactional execution for creating, renaming, moving, and deleting folders and files inside the vault; link and attachment rewriting for moved files; cross-stack boundary detection with primary stack transitions and `referenced_by` adjustment; rollback on failure with undo snapshot support; strict refusal of hostile names, escape sequences, and protected roots (TASK-0203).
- Specification of the diary: comprehensive specification in `docs/specs/diary.md` defining daily entry formats, 13 standard self-assessment scales with exact wording, user-defined scales in TOML, bidirectional run and stack links, and non-intrusive post-rehearsal prompts (TASK-0170).
- Specification of folders and items views: specification in `docs/specs/vault-views.md` defining the Obsidian-modelled Folders view and structured Items view with toolbar actions, stack vs normal folder distinctions, and keyboard navigation (TASK-0202).
- Specification and architecture decision for Markdown-to-PDF export: `docs/specs/pdf-export.md` and `docs/adr/0012-markdown-to-pdf-writer.md` selecting an in-house layout engine on `lopdf` under Rule P10 (TASK-0228).
- Export formats TOML and RON next to JSON: export the whole library or a stack to JSON, TOML or RON with identical data (items, references, and in advanced mode per-stack scheduling values: source, difficulty, familiarity, due date, average, and review history); all three formats round-trip back through import and are parsed and validated with standard ecosystem crates; the export dialog displays re-importability notes (TASK-0220).
- Mark a folder as a stack or as a normal folder: a normal folder can be marked as a stack (creates `_stack.md` with a new id and adopts items), and a stack can be turned into a normal folder via the stack page's More actions; unmarking opens a confirmation modal detailing all changes (settings, status, goals, subjects, sub-stacks, referencing items, and target stack/Inbox); the change can be undone via toast; the Create dialog provides a "Mark as stack folder" choice to create normal folders (TASK-0217).
- Stacks & references in the interface: the item page shows the primary stack and the stacks that reference the item, with Add to stack and Remove from stack buttons and the schedule, due date and familiarity of each; items can be added to another stack from the library bulk bar or by dragging onto a stack; stack pages (library table, backlog, board, roadmap) mark only-referenced items with `(from {primary})`; the Create dialog keeps the parent stack as primary and offers an optional "Also in" picker; every control has an accessible name and keyboard navigation (TASK-0213).
- An item referenced by a second stack counts in that stack's summary, sub-stack tree, reports, familiarity report, assistant context and runs (a run over either stack includes it, and a run over both queues it once); the library has an Also in column; JSON export and import, CSV (an `also_in` column when needed) and imports carry the references and create a stack they name (TASK-0212).
- An item can say which other stacks reference it (`referenced_by`, a list of stack ids in its front matter); the index has a table and a view of which items belong to which stack, and the vault check reports a reference to a stack that does not exist, to the item's own stack or listed twice. Vaults without the key read as before (TASK-0211).
- Run a standard prompt through the connected assistant: choose a prompt and a text file, read exactly what would be sent, press Send, read the answer and import its CSV through the preview; if the preview finds problems, "Ask the assistant to fix these" prepares a message with those rows, sent only after Send (TASK-0207).
- Code blocks with a language (python, r, rust, php, java, cpp, c, toml, json, html, xml, csv, tsv) are coloured: keywords, strings, comments and numbers, in colours that read well in both themes; other languages are drawn plain (TASK-0214).
- Text macros: `{table}`, `{table:3x4}`, `{toc}`, `{code}`, `{code:python}`, `{quote}`, `{image}` and `{color:red}` turn into Markdown when the closing brace is typed; the `{ }` button in the toolbar lists them (TASK-0198).
- Tables (pipe tables with column alignment), underline, text colours from a small palette that works in both themes, strikethrough and an automatic table of contents (a line `[TOC]`) in the Markdown preview, the item page, the cards of a run and help; the toolbar has Table, Underline, Strikethrough and Colour. Underline and colour are kept as the small HTML other readers understand; any other HTML stays visible as typed (TASK-0197, TASK-0215, TASK-0216).
- Six standard prompts for any AI chat (flashcards, questions and answers, a whole stack, notes, summary, critique) that tell the AI exactly which CSV to write, for attached text, PDF text, source code and data files; the Assistant screen has a Prompts tab to copy or save them (one or all), and the import dialog offers a prompt next to the CSV format. Their worked examples are tested against the importer (TASK-0205, TASK-0206).
- CSV and TSV import with column mapping: the delimiter, the encoding (UTF-8 or Latin-1), a header or none and which column is the front, back, title, tags, stack and kind are detected and can be changed under the preview, which shows the first rows; a two-column file is front and back; a kind column can make notes and questions; every problem is listed with its row and column, and "Copy the problems for the AI" puts a ready message on the clipboard (TASK-0208).
- The import dialog lists the formats it reads with the layout each expects, an example and a Save example button; an unreadable file's error names the expected layout; the help has an Import formats page generated from the same table (TASK-0209).
- The editor saves by itself when a word is finished (a space or line break is typed), when something other than text changes, and at the latest 5 seconds after the first unsaved edit; a pause alone saves nothing. Leaving, opening another item and exiting save first; a file changed outside StudyFrog is never overwritten (Reload from disk or Keep mine); a failed save is shown and retried (TASK-0200).
- Leaving an item you changed shows a "Saved <title>" toast that goes away after about three seconds (TASK-0201).
- The notice after Create has an Open button that opens the new item, and the same entry in the notification list keeps that link (TASK-0199).
- `studyfrog --screenshot FILE --theme dark|light|system` takes the picture in a theme, whatever is saved (TASK-0226).
- Colours in the views come from the theme: the calendar's appointments and the graph's notes use a new theme colour, the notification badge the palette's red, and a test fails when a view writes a colour of its own (TASK-0226).
- Appearance settings (Settings > Appearance, the theme also in the profile menu): dark, light or follow the system; the interface size (70 to 200 percent, also with Ctrl and plus, minus and 0) and the text size; compact, comfortable or spacious density; six accent colours; high contrast; reduced motion; the screen to open first (Home, Library or the one used last); the sidebar folded at start; and the window's size and place remembered. Every change shows at once and is saved at once, with no restart, and the saved look and window are used from the first frame. The settings are in an `[appearance]` table of `app.toml` (`docs/specs/appearance.md`) (TASK-0221 to TASK-0225, TASK-0227).
- Navigation history: the top bar has Back and Forward arrows and a History list (the screens, stacks and items visited, newest first, the current one marked), and Alt+Left and Alt+Right do the same; unsaved changes in the editor ask you to save or revert first (TASK-0196).

## [0.3.1] - 2026-11-03

### Fixed
- Scroll bars reserve their own width, so widgets and tables next to them no longer run under them (TASK-0193).
- Rows of a name and a bar (priorities, ratings, familiarity spread) are aligned: the names share a column and every bar starts at the same place and has the same length, with its number on it; the numbers of Types and the due rows on Home line up as well (TASK-0192).
- Every screen has a wider margin (20 px at the sides, 14 px at the top) and the stack page no longer sits against the stack list (BUG-0002).
- The Create dialog's header is two rows, the window controls on one and the stack and type on the other, so the type name is no longer cut off in the normal-width dialog (BUG-0003).
- The four figures at the top of Home (items ready, streak, reviews, items) are spaced further apart (BUG-0004).

### Changed
- StudyFrog draws its own title bar: the top bar moves the window when dragged (the window manager does the moving, so snapping works) and maximises it on a double click, with large minimise, maximise and a red close button (which replaces the separate Exit button and asks about unsaved changes); the edges resize the window (TASK-0190).
- A calmer, easier to read layout: a reading column for the rehearsal setup, Settings and help; grouped cards and captions instead of large headings; the rehearsal setup keeps its Back and Next buttons in view; the library's search is one row and its stack list has a fixed width; the stack page has one primary button and a "More actions" menu, and a two-column Summary (TASK-0192, `docs/specs/ui-calm.md`).
- The status bar no longer has a Help button; the `?` in the top bar, Help in the sidebar and `F1` remain (TASK-0192).

### Added
- The rehearsal screen has a full-width progress bar with "N done · M left" counted against the items still open: an item with its answer showing is still open until it is rated or skipped, and items put back to repeat count as open and are named (TASK-0194).
- `studyfrog --screenshot FILE [--screen NAME] [--size WxH]` saves a PNG of a screen of the running app and exits, for looking at the real rendering (`home`, `library`, `rehearse`, `run`, `settings`, `help`, `calendar`, `board`, `reports`, `agent`, or `stack:<folder>`) (TASK-0192).
- Cancel, the close button, Escape and a click outside the Create dialog ask "Discard what you wrote?" when a field has content or a changed value; an untouched dialog closes at once (TASK-0188).
- A red button at the right end of the top bar ("Exit StudyFrog") closes StudyFrog and asks first when the editor holds unsaved changes (TASK-0189).
- The Import tab lists what it finds on the computer: Obsidian vaults (from Obsidian's own list, with the number of Markdown files), Anki packages in Downloads, Documents and Desktop, and Anki profiles with the hint to export a package; "Use" shows the preview for the chosen one and nothing is read before (TASK-0191).
- A proposal for a calmer UI in `docs/specs/ui-calm.md` for the owner to accept (TASK-0192).

## [0.3.0] - 2026-11-03

### Fixed
- Teacher and subject names may contain commas ("Prof. Dr. Künkelmann, Harry"): a stack's `subjects` list is written as a
  quoted list when needed and read back whole, and the Settings screen refuses a name that cannot be stored before saving,
  keeps what was typed and says why next to the field (BUG-0001).

### Changed
- The `?` in the top bar opens the help page of the screen instead of a menu of shortcuts (they are on the Keyboard shortcuts page) (TASK-0162).
- The Create dialog is laid out like Jira's: the parent stack and the type (a drop-down) in the top left corner, a large title, the Markdown editors, then priority, difficulty and tags (due date for a stack); "Add PDF ..." moved into a More menu, and Enlarge and a grip at the lower left widen the dialog (TASK-0186).
- Every Markdown editor, in the dialog and on the item page, gets a bar under it that makes it taller when dragged or with the arrow keys (TASK-0187).
- New cards, notes and questions start with priority medium and difficulty difficult (the highest) unless the user chose others; stacks get neither (TASK-0185).
- The library page no longer holds the new-card form: a card, note or question is made in the Create dialog, and the import panel and the card types of extensions moved into an "Import…" dialog on a stack's list (TASK-0134).
- The bell's notifications are records the user manages: each has an id, time and read state and is kept in the vault's database for 30 days
  (at most 200), so they survive a restart. Opening the panel no longer marks everything read (TASK-0126).
- The top bar shows the open vault's folder name next to StudyFrog (cut at 24 characters, full path in the tooltip), reversing the
  removal of STORY-0041 at the owner's request (TASK-0128).

### Added
- Help inside the app (`F1`, the `?` in the top bar, or Help in the sidebar, with or without a vault): a page tree (Start, Screens, Reference, Your pages, Tutorials), the page for the screen that is showing, links between pages, Back and "Back to" the screen, and a search over titles and text. The 26 supplied pages are compiled in from `docs/help/` and read-only; a test fails when one links to a page that does not exist (TASK-0161, TASK-0162).
- The help pages themselves: Why StudyFrog, Getting started, Privacy and your data, a how-to for every screen, the FAQ, the glossary, keyboard shortcuts, the query language, familiarity and statuses (TASK-0165).
- Own help pages: New page, Copy to my pages (from a supplied page), Edit, rename and Delete; each is a Markdown file in the vault's `wiki/` folder (not indexed as items, part of backups), searched and linked like the supplied pages (TASK-0163).
- The assistant can read the user's help pages ("Also read my help pages" on its wiki tab, shown in the preview before Send) and propose new pages or edits; nothing changes until the user applies them, and supplied pages are never sent nor offered for change. Help and the assistant link to each other (TASK-0164).
- A "First steps" tutorial: offered once after the first vault opens (dismiss to skip; `tutorial_offered` in `app.toml`), repeatable from Help; a card at the lower right walks through creating a stack and a card, rehearsing and planning a session, opening each screen and marking a step done when the user has done it (TASK-0166).
- An import and export dialog (from the Create dialog's More menu, a stack's page and Settings under Data): Export asks for the mode (Default without scheduling, or Advanced with the review history), the format (JSON zip, CSV or Markdown zip) and the folder; Import picks a file or folder, shows a preview with counts, warnings and each item's verdict, lets the user skip, replace or keep both for duplicates, and writes nothing on a cancel (TASK-0160).
- Importers for Anki (`.apkg`: decks to stacks, basic and cloze notes to cards, tags and pictures kept, the review log imported as reviews with the schedule rebuilt on request) and Obsidian (a vault folder or zip: folders to stacks, Markdown files to notes with front matter, wikilinks, tags and pictures; callouts, dataview blocks, comments, highlights, block references and note embeds stay as text and are counted for the preview), read through a table of format adapters that `docs/EXTENDING.md` shows how to extend (TASK-0167, TASK-0168, TASK-0169).
- Export and import engine (`docs/specs/interchange.md`): the library or one stack as JSON (versioned, with the review history only in the advanced mode) with pictures and PDFs in a zip, as CSV, or as the vault's Markdown files in a zip; import reads them into a bundle, a dry run reports what would be created, what exists and what fails, and duplicates are skipped, replaced or kept as a second copy. The existing card import runs through the same path. Hostile paths and files are refused (TASK-0157, TASK-0158, TASK-0159).
- The review after a run and a new Runs tab in the reports show each run's mode, pacing, planned time (countdown), time used and answers a minute; the Runs tab filters the history by mode (TASK-0182).
- Run setup offers a time for the run: no clock, a countdown (5, 10, 15 or 25 minutes or any from 1 to 180) or a count up. The rehearsal screen shows the time left or used, says "One minute left" and "Ten seconds left" in text and announces them to screen readers, and a countdown ends the run by itself at zero, leaving the item on screen unrated and opening the review (TASK-0181).
- The stack page has Board (a column per status, items move by dragging or a menu), Calendar (the month grid, day list and planned reviews of that stack only, sharing the global calendar's code) and Reports (the stack's familiarity and least familiar items) tabs; tabs that do not fit go into a More menu (TASK-0156).
- The stack page has a Roadmap tab: a row for each stack inside it and each item directly in it, with a bar from its first date to its next one on a time axis with a line for today; zoom (Today, Weeks, Months, Quarters), scrolling both ways, only the rows in view are drawn (1000 rows measured), and each bar is a named button that opens its stack or item (TASK-0155).
- The stack page has a Goals tab: a Markdown bullet list (check boxes allowed) edited in place and saved under a `## Goals` heading in the stack's `_stack.md`, where it is searchable and part of backups; the rest of the file is kept (TASK-0154).
- The stack page has a Backlog tab (Up next and Not yet planned with the count per status; an item moves between them by dragging or by a button, which makes it Learning or puts it on hold) and a Sub-stacks tab (the tree of stacks inside, folding, each with status, progress, due date and familiarity, opening one on a click) (TASK-0152, TASK-0153).
- A stack opens as a page like Jira's project view: a breadcrumb, the stack's title, a status chip that changes the stack's status, and the actions Details, New item, Import and Start rehearsal, with tabs Summary and List (the last tab is remembered per stack; Left and Right switch). The old stack page is the List tab (TASK-0150).
- The Summary tab, read only when it opens, shows cards for completed, updated, created and due-soon items, a status ring, priorities, types, an activity feed and the sub-stacks with status, progress and familiarity, all with their numbers as text (TASK-0151).
- Settings (Study tab) tells from the user's own review history whether familiarity improves the predictions of recall: a replay scores plain FSRS against the familiarity-adjusted prediction by log loss and a calibration table, fits the strength and half-stability on the first 80 percent when there are at least 1000 reviews, and shows the verdict, the best values and the change in reviews; it never switches the factor off itself (TASK-0184).
- A Familiarity tab in the reports shows how the items spread over the scale and, per stack and per subject, the roll-up and the three least familiar items with the inputs behind each value; the item page has a Familiarity card with the value, its drivers and its course over time (TASK-0138).
- Home shows a "Needs attention" card with the five least familiar items and why (low familiarity, overdue, an exam within 14 days); run setup offers "Least familiar first" and a familiarity filter (TASK-0139).
- Queues use familiarity: under a daily allowance the least familiar due items are chosen first, `familiarity_threshold` in `studyfrog.toml` leaves out due items at or above it, and a run can be ordered "Least familiar first" or limited to items less familiar than a level (TASK-0140).
- The calendar shows the reviews the schedule planned as chips ("French: 4 of 10 due") told apart from sessions: a click opens the stack, a chip can be dragged to another day, and the side panel and the context menu move it by date for those who do not drag; "Undo last move" takes a move back and a move past an exam or due date says so. "Show the reviews the schedule plans" turns them off (`calendar_reviews = false` in `app.toml`) (TASK-0148).
- Planned reviews can be read from the schedule and moved to another day (`review_plan` in the service): a move changes only the due date of the items, warns when the new day is after an exam or deadline of their stack, and can be undone (TASK-0147).
- The calendar crate has a planner (`review_plan`) that turns the schedule into entries, one per stack and day with the item count and the stack's size; done and paused items and stacks make none (TASK-0146).
- Library rows can be ticked (a check box per row and one for all rows the filter lets through); a bar then sets status, priority or difficulty, moves the items to a stack, adds or removes a tag, or deletes them after a question that names the number. Each change is one step that "Undo last change" takes back (TASK-0133).
- Saved filters: the Filters menu saves the current filter or query under a name in `filters.toml` in the vault, reopens it and removes it (TASK-0133; per vault, not per profile, as the library spec proposes).
- The library's search bar has a Standard mode (title word, type, status and priority) and a Query mode with the errors under the box, suggestions for field names, operators and values, and a switch that turns the fields into a query and a plain query back into fields (TASK-0132).
- The library list: every item of the vault in a table with sortable columns (title, type, status, priority, difficulty, familiarity, stack, subjects, tags, reviews,
  due, created, updated, path; a click sorts, a second click reverses, shift adds a key), a column menu that is remembered, 'N of M items', and a tree entry 'All items';
  10 000 rows draw in about 11 ms per frame because only the rows in view are built. The search is still the title search (TASK-0131).
- The library's query language: a parser with messages that name the column, a printer, and an evaluator over item rows (fields for text, type, status, stack, subject,
  teacher, tag, priority, difficulty, familiarity, due, reviews, dates and path; `AND`, `OR`, `NOT`, `IN`, `IS EMPTY`, `ORDER BY`). User text never reaches SQL; 10 000 items are
  filtered in a few milliseconds. The screens come with TASK-0131 and TASK-0132 (TASK-0130).
- Answer speed counts for familiarity and scheduling: each rating stores how long its answer was on screen and how long it should take to read (schema version 17),
  views store the reading time of the item, and a card read faster than expected gains more familiarity than one read slowly. Gains close to an exam or due date of the
  stack count for less (half on the day itself, in full from 14 days before), at the owner's request (TASK-0137, TASK-0141).
- The rehearsal engine knows two more pacings besides the free run: a countdown that ends the run by itself when its active time is up (the item on screen
  counts as not answered, with no rating and no schedule change; a pause stops the clock) and a count up. The pacing is stored with the run (schema
  version 16, older runs read as free) and a `time_up` event marks the end of a countdown. The screens come with TASK-0181 (TASK-0179, TASK-0180).
- Specifications of the library redesign (`docs/specs/library.md`: a Jira-style list with columns, sorting, selection and bulk actions) and of the
  query language (`docs/specs/query.md`: grammar, fields, operators, error messages, limits) (TASK-0129).
- A notification center under the bell: filters for all, unread and errors, mark one or all read or unread, delete one or clear all, the list
  grouped under Today, Yesterday and the date, a link on notifications that have a target (reminders open the calendar), and an unread count on
  the bell (TASK-0127).
- Familiarity values and their use in scheduling: `studyfrog_report::familiarity` values an item from its ratings, views, mark and the fall without
  contact; after each rating the next due date is multiplied by a factor that compares familiarity with what the FSRS stability suggests (0.25 to 2,
  nothing before the third review, never stretched past an exam or deadline, the stored state untouched). On by default; `familiarity_strength` in
  `studyfrog.toml` (0 is off) and `familiarity = false` in a `_stack.toml` switch it. Validation numbers are in `docs/specs/reports.md` (TASK-0137,
  TASK-0141).
- The item page times how long an item is looked at: active time only (a minute without input stops the clock), one `view` record per
  visit and an `edit` record when the item was changed, kept in the database and in backups. Settings, under Study and scheduling,
  can switch it off (`track_views = false` in `app.toml`) (TASK-0136).
- Statuses on screen: a status menu on the item page (Automatic, Not studied yet, Learning, In review, Complete, On hold), the status
  on every item in a stack's list, and a line under a stack's heading with how many of its items, sub-stacks included, are in each
  status (TASK-0145).
- Item status in the data: the optional `status` front-matter key (`new`, `learning`, `reviewing`, `done`, `paused`) on every kind, an
  index column (schema version 12) and an effective status on every item list, derived from the review history when the key is
  absent. The screens come with TASK-0145 (TASK-0144).
- Specification of the familiarity value (`docs/specs/familiarity.md`): update rules for ratings, views and view time, the slow fall
  and when it pauses, notes, the stack roll-up, confidence, worked examples and the properties to test (TASK-0135).
- Settings are in tabs by domain (Study and scheduling, Rehearsal, Answers and AI, Data, Subjects and teachers); the last tab is
  remembered in `app.toml` as `settings_tab`, and the Left and Right keys move between tabs. Tabs for General, Appearance and
  Notifications appear when they have settings (TASK-0142).
- Specification of item statuses (`docs/specs/status.md`): five statuses, where they are stored, who may change them, what they do to
  the schedule, and how a stack rolls up its items (TASK-0143).
- Priority order for 0.3.0 in four waves in `docs/roadmap/M10-usability-and-insight.md`; wave 1 is `ready` on the board (BUG-0001,
  TASK-0128, TASK-0135, TASK-0142, TASK-0143).
- Research and a proposal for combining familiarity with FSRS: a relative factor on the FSRS interval that leaves the stored
  state untouched, a reproducible simulation (`docs/research/familiarity-fsrs-sim.py`) and a replay that tells each user whether it
  helps; it is on from the start (TASK-0183, TASK-0184).
- Planning for 0.3.0 (phase M10, EPIC-023): notification management, the vault in the bar, a library with an SQL-like
  search, a stack overview with sub-views and a separate stack detail view, a familiarity value used in reports and scheduling, tabbed
  settings, item statuses, calendar entries for the next reviews that can be moved by drag and drop, import and export
  with Anki and Obsidian importers, a built-in wiki with FAQ and tutorials, a diary and rehearsal modes for countdown and count up, recorded as STORY-0055 to
  STORY-0070, BUG-0001 and TASK-0126 to TASK-0184; the issue checker accepts phase `m10`.
- Releases carry a `.tar.gz` (binary, desktop entry, icon, licences, README, changelog, release notes) and its `.sha256`
  next to the AppImage; the packaging script writes them and the release workflow uploads them (TASK-0125).

## [0.2.0] - 2026-10-31

### Added
- Links inside a PDF: outlined on the page, page links followed with a click, web and mail addresses opened only after
  the user presses Open on a question that shows the address; launch, script and other actions are never run (TASK-0123).
- Documentation, tests and benchmarks for PDF support: `docs/specs/pdf.md`, headless view tests with the accessibility
  audit, a 500-page benchmark with budgets, and draft release notes `docs/RELEASE_NOTES_0.2.0.md` (TASK-0122).
- PDFs in the library: the stack page lists its PDFs with page counts, 'Add PDF ...' (also in the Create dialog) copies a
  file into the stack under a safe name, renaming a PDF moves its notes file and rewrites every note, card and link that
  names it, `[text](file.pdf#page=3)` and `![](file.pdf#page=3)` open the viewer at that page, and the home page lists the
  PDFs that changed most recently (TASK-0121).
- Cards and questions from PDFs: a highlighted passage or the selected words become a card or a question with a link to
  the page, shown as a 'Source' link while rehearsing; the assistant can read chosen PDF pages (shown first, sent only
  after Send) and propose cards and questions that name the page they come from (TASK-0120).
- Pictures from PDFs: save the pictures of a page or a dragged rectangle of a page (figures, formulas) into the vault's
  assets folder under the usual picture rules, and put them at the end of a note made from the page (TASK-0119).
- Notes from PDF text: the text of pages or of a selection becomes a Markdown note in a stack, with a `source` key that
  leads back to the page; the item page offers 'Open source' and the viewer lists the notes made from the file
  (TASK-0117). Pages that are only pictures are flagged in the index and the viewer, and notes made from them say so
  (TASK-0118). `docs/specs/pdf.md` describes the formats and limits.
- Highlights and notes on PDFs: choose a colour, highlight the selected words or place a note, edit and delete them in a
  list, jump to each; stored in plain text in `name.pdf.notes.toml` next to the PDF, searchable, included in backups, and
  followed to the right page if the PDF is replaced by a revision (TASK-0114).
- Rearranging PDFs: turn, move and drop pages in a thumbnail grid with undo, append other PDFs, split by page ranges and
  save as a new file with new title, author and subject and, if wanted, the highlights and notes written into the copy
  as standard annotations; the original is never changed (TASK-0115, TASK-0116).
- PDF files in the vault are indexed (title, page count and the text of every page, extracted once per change) and the
  global search finds them by title and by text, showing the page of the hit (TASK-0111).
- The PDF viewer: one page at a time with zoom (fit width, fit page, percent), page jump, outline, page thumbnails,
  search inside the file with highlighted hits, word selection by dragging and copy, and the page's text for screen
  readers; pages are drawn on the worker thread and cached (TASK-0112, TASK-0113).
- The `studyfrog-pdf` crate (TASK-0110, TASK-0113): reads page sizes, outline, properties, words with boxes, text,
  Markdown and pictures; draws pages and regions; rotates, drops, reorders, merges and splits pages; writes
  highlights and notes into a copy. Size, page, pixel and time limits and crash containment keep hostile files from
  taking the app down.
- PDF library spike (TASK-0109): measured findings in `docs/research/pdf-library-spike.md` and a proposed ADR 0010
  (hayro to draw pages, pdf_oxide to read and edit, both behind a `studyfrog-pdf` crate); ADR 0010 is accepted.
- Planning for PDF support: EPIC-022 with STORY-0054 and TASK-0109 to TASK-0122 (view, annotate, rearrange and extract
  PDF files), a new phase M9 in the roadmap and the `m9` label.

## [0.1.0] - 2026-10-31

### Added
- The Assistant screen: connect a local model (Ollama and similar) or a remote one (OpenAI-compatible, Anthropic),
  let it read chosen stacks and propose changes like an LLM wiki editor (new notes, cards, questions, edits, an
  index) that you accept one by one, and be examined by it with the verdicts recorded as reviews only when you say
  so. A remote model is refused until you allow it; keys live in a private file in your config folder; calls are
  logged without their content (STORY-0049, ADR 0009).
- Backups and restores (Settings, and the welcome page): one plain `.tar` with the vault's files, a consistent copy of
  the database (review history, runs, the board and the calendar), and optionally the settings and API keys. A
  restore goes into a new or empty folder, never overwrites anything, and keeps the old settings next to the new
  ones (STORY-0050).
- User profiles: each has its own settings, recent vaults and API keys; switch, create and delete them from the
  profile menu (or the welcome page). The vault path is no longer shown in the profile menu (STORY-0051).
- Pictures in notes, cards, questions and answers: a Picture button in the Markdown toolbar copies a png, jpg or gif
  into the vault's `assets/` folder and inserts `![alt](assets/name.png)`; pictures show in the editor preview, the
  item page and while rehearsing, and only pictures inside the vault are loaded (STORY-0044).
- A Performance tab on the reports screen: reviews, items reviewed, good-or-easy share, answer times, rating bars
  and a 30-day chart for the whole vault, a stack, a subject or one item, over all time, 30 or 7 days, drilling
  down by clicking (STORY-0048).
- The item page: title, a Markdown editor for every text area (toolbar, Write, Split or Preview), and a side column
  with the kind, stack, subjects, priority, difficulty, tags, dates, the links to the item and the items that link
  to it (STORY-0042, STORY-0043). The Create dialog uses the same editor, wide for notes.
- Priority (lowest to highest) and difficulty for stacks, notes, cards and questions, inherited from the stack;
  priority orders spaced runs and breaks ties in the other orders, and the owner's difficulty stands in for the
  memory difficulty of new items (STORY-0053).
- Teachers and subjects (`subjects.toml`, edited in Settings); stacks name subjects and what is inside takes them
  (STORY-0045).
- A date picker (a month grid under the date field) in the Create dialog and the calendar form; a stack's due date
  can be set or changed from its Library page (STORY-0046, STORY-0052).
- Stacks whose due date has passed are left out of the suggested run and the home page's counts; choosing such a
  stack by hand in the run setup still works (STORY-0052).
- Cards made by an import or by the Library card form also get a board card (the first 25 of an import, then one
  summary card); board cards are draggable by their whole face and columns light up as drop targets (STORY-0047).
- ADR 0009 (accepted): how an AI agent will work the vault, where its keys and logs live, and that it only
  proposes changes (STORY-0049). New backlog stories for backups, profiles, priority and difficulty
  (STORY-0050, 0051, 0053).
- The end-of-run review is a summary with tiles and bars and a scrollable card for every item; prompts and answers
  are drawn as Markdown while rehearsing (STORY-0040).
- The vault's folder name and path no longer appear in the top bar, the home page, the status line or an "Opened"
  notice; the status line shows the size of the vault instead (STORY-0041).
- Calendar entries are now study sessions, tasks, appointments, exams or deadlines, each with up to five reminders
  that show once as a toast and a notification while the app is open; the month grid fills the window with
  coloured entries and a "Coming up" list (TASK-0105).
- A stack can be given a due date and reminders in the Create dialog, which puts a deadline on the calendar; every
  item made in the dialog also gets a card in the first board column (TASK-0106).
- The run setup is three steps: What, How, Review (TASK-0107). The home page lists the items due now and the next
  seven days and fills the window (TASK-0108).
- `docs/specs/UX_EVIDENCE.md`: the usability research behind these changes, with how strong each finding is
  (TASK-0104).
- A home page (greeting, items ready today, streak, stacks waiting with a Rehearse button, planned sessions,
  recent items and runs) that a vault opens on; a shared theme (palette, text sizes, spacing) for every screen;
  stacks as cards and items as rows in the library; a card layout, progress bar and coloured rating buttons in
  the rehearsal screen (TASK-0100 to 0103).
- A Create dialog (modal) for stacks, cards, notes and questions with a parent-stack picker, tags and "Create
  another"; stacks can sit inside other stacks (shown as a tree in the library, and run setup includes the stacks
  inside a ticked one); first launch shows a welcome page and a clickable in-app notification that opens the
  vault dialog, instead of a path field (TASK-0096 to 0099).
- A top bar (sidebar toggle, vault switcher, brand, global search, Create menu, notifications, help, settings,
  profile) and a foldable sidebar for the screens; global search over item titles; a notification list behind
  the bell (TASK-0093, 0094, 0095).
- Extensions: `studyfrog-ext` (importers and card types as plain data, a registry), an example extension
  (tab-separated importer, cloze card type) that changes no core crate, a card-type chooser and an "Import from
  a file" panel in the library, `docs/EXTENDING.md` and ADR 0008 (TASK-0063, 0064, 0065).
- Packaging: `scripts/package-appimage.sh`, a desktop entry and icon, and a CI workflow that runs the gate and
  builds the AppImage (TASK-0067). The 0.1.0 AppImage builds and starts locally; the workflow has not run yet.
- Choose the vault folder with the native dialog (through the desktop portal; the app binary supplies it, the
  UI crate does not link the dialog library) (TASK-0092).
- Shuffling is its own switch: any order, spaced repetition included, can be shuffled without changing which
  items are chosen or whether the schedule is updated (schema v7; runs stored as `shuffled` become sequential
  with the switch on) (TASK-0091).
- Open flags closed: answer runtimes are calibrated on a labelled set (word matching gets a pivot of 0.44;
  the language model keeps 0.7); typed-answer scores are stored with their run (schema v6) and shown in the
  review; `answer_engine = "words"` and a Settings choice force word matching; the user's easy/difficult mark
  suggests a rating; the fast/slow/minimum-words thresholds are configurable in `app.toml` (TASK-0086 to 0090).
- Reports: retention by time since the previous review, time to answer (percentiles, histogram, by the
  difficulty mark), and a 30-day review-load forecast with a backtest on real or simulated history; a Reports
  screen; each report computes in under 10 ms on 100,000 reviews (TASK-0059, 0060, 0061, 0062).
- Achievements: rules as data (built-in set or the vault's `achievements.toml`), statistics from the review
  history including streaks, unlocks remembered once (schema v5), an Achievements screen with progress and
  an announcement when something unlocks (TASK-0056, 0057, 0058).
- Reading speed can be measured on the Settings screen (read a lorem ipsum text, press done) and is kept in
  `app.toml`; rehearsal uses it as the baseline. Reading much slower than expected now suggests a harder
  rating (Good to Hard, Easy to Good) (TASK-0084, 0085).
- Rehearsal: mark an item Easy, Medium or Difficult while it is on screen (buttons or E/M/D); reading speed
  against a 238 words-per-minute baseline suggests a rating; the post-run review shows previous rating,
  own mark and rating and the suggestion, and can apply it, rebuilding the item's schedule (TASK-0081,
  0082, 0083).
- Semantic answer comparison: a candle MiniLM runtime (`studyfrog-candle`, linked by the app only) used
  automatically when `all-MiniLM-L6-v2` is installed under the app data directory, with fallback to word
  matching and the reason shown; the result line says which engine compared the answer. Measured: p95
  about 75 ms per comparison, about 85 MB of memory (TASK-0050, 0051).
- Answer comparison: `SlmRuntime` trait, scoring over embeddings (best of reference and accepted answers,
  rubric points, verdict bands, suggested rating), an in-house offline lexical runtime, property tests and a
  benchmark; typing an answer in rehearsal now shows the match, the verdict and a suggested rating
  (TASK-0049, 0052, 0053, 0054, 0055).
- Calendar screen: month grid with recurring sessions, day panel (edit, delete, skip a day, start a
  rehearsal for the session's stack) and a session form (TASK-0045).
- Board screen: columns with WIP counts and over-limit marking, drag and drop plus push/pull/up/down buttons,
  card editor with stack link, optional forcing past WIP limits (TASK-0048).
- Planning foundations: `studyfrog-calendar` (sessions, recurrence checked against a day-by-day oracle) and
  `studyfrog-board` (columns, WIP limits, strict/forced moves, push/pull, commands; property-tested); schema v4
  (sessions, board); service use cases for both; `core::civil` (dates, weekdays, month grid, local time); a
  local time-zone provider so daily limits and run previews use the local day (TASK-0044, 0046, 0047, 0078,
  0079, 0080).
- Link graph screen: the wikilink graph as a force-directed picture with pan, zoom, hover, click-to-open, a
  re-run button, an accessible item list, and live refresh that keeps node positions. New pure crate
  `studyfrog-graph` (SoA layout, uniform-grid repulsion, deterministic, zero allocation per step): 5k nodes /
  10k edges step in 4.0 ms; verified allocation-free with dhat. Items now show when they are next due
  (TASK-0042, TASK-0043). EPIC-009 and phase M4 complete.
- Editor: open any item from the browser, edit title, tags and text areas with a live Markdown preview (headings,
  emphasis, lists, task items, quotes, code, wikilinks and URL links), undo/redo, save with conflict detection
  (a file changed elsewhere is never overwritten), revert/reload, unsaved-changes guards, clickable wikilinks.
  `studyfrog-md::blocks` (preview model, pulldown-cmark), `studyfrog-service::document`, `resolve_title`
  (TASK-0040, TASK-0041).
- Rehearsal screen: keyboard-first (Space, 1-4, S, P, Esc), pause that hides the question, flip and type-answer
  modes, look-ahead content loading, end-of-run summary; every step saved through the service (spaced runs update
  the schedule, practice-only runs do not). Service `start_run`, `load_item_content`, `StepContext`; crashed runs
  are recovered when a vault opens. 15 tests (TASK-0039). EPIC-008 (UI shell) complete.
- Run setup screen: presets (spaced, everything in order, shuffled, hardest first, not seen for longest),
  all orders, filters (kind, stack, tags, state, difficulty), limit, requeue, and an explicit "update my
  schedule" switch; live count of matching items from the index. 6 tests (TASK-0073).
- Stack browser: choose a stack or the inbox, filter its items by title, create stacks and cards from the UI.
  Service use cases `create_stack`/`create_card` (never overwrite, validated, reindexed at once), in-house ULID
  composition with OS randomness, `list_items`; 14 new tests incl. a real-vault build-a-stack flow
  (TASK-0038).
- First desktop app (EPIC-008 so far): eframe/egui window with a vault chooser (remembers the last vault),
  library overview (stacks, counts, inbox, problems), navigation and settings view; pure
  `update(State, Msg) -> (State, Vec<Effect>)` with a worker-thread executor and 1 s change polling;
  21 tests incl. headless egui view tests and an accessibility audit; `--smoke-frames N` for CI
  (TASK-0036, TASK-0037).
- `studyfrog-service::session` (open a vault, index it, poll it), `studyfrog-store::overview` (stacks, counts,
  problems), per-user `app.toml` (`studyfrog-config`, `studyfrog-store::app_config`), `poll::prime`.
- `studyfrog-config` (ADR 0007, Proposed): pure parsing and rendering of `studyfrog.toml` (vault defaults:
  scheduler, retention, interval cap, FSRS weights) and `_stack.toml` (scheduler, retention, daily limits);
  unknown keys are errors with line numbers (TASK-0075).
- Settings pipeline: `_stack.toml` is indexed (`stack_identity`, `stack_config`, view `stack_setting`), loaded into
  the library, and included in polling; schema v3 (TASK-0075).
- SM-2 in the service: each item is scheduled by its stack's scheduler (default from `studyfrog.toml`),
  stack retention overrides apply to FSRS, switching a stack keeps items scheduled (TASK-0076).
- Per-stack daily new/review limits for spaced runs, counted from the review history with an injected day window;
  practice-only runs do not count (TASK-0077).
- Run filters are persisted with the run and round-trip exactly (TASK-0074). EPIC-018 complete.
- `studyfrog-service` (application layer, ADR 0006 Proposed): `apply_rating` (pure), `record_step`,
  `begin_run`, `recover_runs`, `new_run`/`candidates`; end-to-end tests over a real vault and database
  (spaced run updates the schedule; practice-only run leaves it untouched; due items return most-overdue
  first; crash recovery) (TASK-0033).
- `studyfrog-store`: schema v2 (run plan columns, in-place upgrade tested), run and event persistence
  with crash recovery, review and FSRS state storage, item positions (TASK-0072, TASK-0033).
- `studyfrog-core`: event name/payload encode and decode. Phase M3 complete.
- Queue builders in `studyfrog-rehearsal`: filters (kind, stack, tags case-insensitive incl. non-ASCII,
  state, difficulty), orders (spaced with daily budgets, sequential, seeded shuffle, hardest/easiest,
  stalest, lowest predicted recall), length cap; in-house SplitMix64 shuffle; 13 tests incl. an
  independent filter oracle; benchmark: engine step 23 ns, 10k-candidate queue 82 us to 3.9 ms
  (TASK-0034, 0069, 0070, 0035).
- Run plans in `studyfrog-core`: orders (due-first, sequential, shuffled, hardest/easiest, stalest,
  lowest predicted recall), filters, schedule effect (update vs practice-only), requeue-on-Again, limit
  (TASK-0068). Rehearsal is not only spaced repetition.
- `studyfrog-rehearsal`: pure run state machine (`step`), injected clock, latency excluding paused
  time, requeue with cap; 9 tests incl. an invariant property (TASK-0030, 0031, 0032, 0071).
- `studyfrog-store`: vault walker, in-house content hash, stack ownership by folder, non-fatal scanner
  with duplicate-id detection, `Library` builder (inbox stack for orphans), incremental reindex
  (stat-only for unchanged files, rename-safe, duplicate-healing), polling change detection with
  settle window; 17 tests and a 10k-note benchmark (cold index 0.57 s). EPIC-006 and phase M2
  complete (TASK-0027..0029).
- `studyfrog-store`: SQLite schema v1 (derived index + authoritative history), transactional
  migration runner, WAL/foreign-key pragmas, 6 tests (TASK-0025, TASK-0026).
- `studyfrog-md`: in-house ULID and RFC 3339 codecs, front-matter fence/key-block splitter, typed
  parser and renderer for stack/note/card/question with verbatim unknown keys, `???` splitter,
  wikilink extraction, duplicate-id check; 25 tests incl. 1000-case round-trip property
  (TASK-0020..0024). EPIC-005 complete.
- `studyfrog-srs`: SM-2 (`sm2_next`, ease/interval rules from the original), `Sm2State` in core, and a
  criterion benchmark (100k cards: FSRS 2.9 ms, SM-2 0.72 ms); FSRS budget tightened to 10 ms
  (TASK-0018, TASK-0019). EPIC-004 and phase M1 complete.
- `studyfrog-srs`: FSRS-6 (`initial_state`, `next_state`, `retrievability`, intervals, Anki-style
  deterministic fuzz); `MemState` in core; golden test reproducing the py-fsrs reference intervals and
  7 property tests (TASK-0016, TASK-0017).
- `studyfrog-core`: `Run`, `RunMode`, `RunState`, `RunEvent`, `EventKind`, `Latency` and persisted-name
  functions, 2 tests (TASK-0015). EPIC-003 (core domain types) complete.
- `studyfrog-core`: `Rating` (persisted values 1-4), `Review`, `rating_value`/`rating_from_value`,
  3 tests (TASK-0014).
- `studyfrog-core`: `Meta`, `Stack`, `Note`, `Card`, `Question`, `UnixMs` and `Library` (one arena per
  kind, no `Item` enum), plus a test (TASK-0013).
- `BOARD.md`: Kanban view generated from issue front-matter by `scripts/board.sh`; WIP limit 3;
  `scripts/check.sh` fails when it is stale.
- `CHANGELOG.md` and `CLAUDE.md` (working rules: keep board and changelog current).
- `studyfrog-core`: `Uid`, typed arena index `Idx<T>`, append-only `Arena<T>` with free
  functions, 7 tests (TASK-0012).
- M1 issues: EPIC-003/004, STORY-0005..0008, TASK-0012..0019.
- `tools/repo-lint` (standalone, `syn` + `serde-saphyr`): inherent-`impl` scan including macro
  bodies, and issue front-matter validation (TASK-0006).
- `scripts/check-issues.sh`, `scripts/check-deps.sh`, `scripts/check-impls.sh`,
  `scripts/linecount.sh`, `scripts/check.sh` (TASK-0003..0005).
- Documentation library: vision, architecture, design principles, glossary, ADRs 0001-0005,
  roadmap M0-M8 with phase files, specs, research, contributing guides (TASK-0007..0011).
- M0 issues: EPIC-001/002, STORY-0001..0004, TASK-0001..0011.
- Cargo workspace with 12 documented, empty crates and shared lints (TASK-0001, TASK-0002).
- `rustfmt.toml`, `clippy.toml`, `rust-toolchain.toml` (1.99.1), `.gitignore`.

### Changed
- The Create button opens the dialog (the dropdown with study session, board card and import entries is gone;
  those are made on their own screens). The inline "new stack" form in the library is replaced by "New stack…".
- The screen buttons moved from the top row to a sidebar; "Close vault" moved into the profile menu.
- The lowest-recall queue order is 7 times faster (recall computed once per item) (TASK-0066).
- The application binary is called `studyfrog`.
- Dev builds use line-table debug info only, so the many test binaries link much faster.
- `md`'s RFC 3339 codec shares `core::civil` instead of carrying its own date arithmetic.
- Daily limits and run creation use the local day (was the UTC day).
- Queues are built from the index (`studyfrog_store::overview::load_indexed_items`) instead of a parsed
  `Library`; `new_run` no longer takes a library.
- Stack settings (`daily_*`, `srs`) are no longer `_stack.md` front-matter; they moved to `_stack.toml`
  (ADR 0007). `StackDoc` now carries only identity, description and body; core `Stack` has `settings`.
- ADR 0005 amendment accepted: no `gray_matter`; in-house fence splitter plus `serde-saphyr`
  (TASK-0021 retitled). New design principle P10: build instead of buy when we can do better.
- ADRs 0002-0005 accepted (GUI: egui/eframe; persistence: rusqlite + Markdown files;
  SLM: candle, gated on M6 spike; Markdown schema).
- ADR 0005: YAML crate decided (`serde-saphyr`); amendment to drop `gray_matter` is Proposed.
- `docs/specs/srs-algorithm.md` pins FSRS-6 with default weights.
- Research citations verified where possible and marked `[v]`/`[u]`; benchmark budgets
  documented as provisional with rationale.
- Default branch is `main`.

### Removed
- Placeholder hello-world package and the unverified `repository` manifest field.

### Fixed
- Graph layout never came to rest because the repulsion jumped to zero at its cutoff (nodes flickered in and out of
  range); a smooth window fixes it. Found by measuring energy, not by a failing test.
- Board/issue bookkeeping: TASK-0016, STORY-0007 and EPIC-004 were still `backlog` after M1. `repo-lint`
  now rejects a parent whose children are all done but is not, a done parent with open children, and a
  done item with unchecked acceptance criteria.
- `scripts/linecount.sh` no longer miscounts the `wc` total line on non-English locales.
