# StudyFrog Features

StudyFrog is a Linux-native, offline-first desktop learning environment built in Rust. It stores all of your learning material as plain, human-readable Markdown files inside a folder on disk (your **vault**) while maintaining a fast local SQLite index (`.studyfrog/index.sqlite`) for spaced-repetition scheduling, search, analytics, and graph visualization.

> Looking for upcoming roadmap items? See **[PLANNED_FEATURES.md](PLANNED_FEATURES.md)**.

---

## 1. Plain-Text Vault & Knowledge Organization

### Vaults & Multi-Vault Switcher
- **Folder-Backed Vaults:** Any directory on your filesystem can be opened as a vault. Notes, flashcards, questions, stacks, and diary entries live as standard `.md` files with YAML-like front-matter that remain readable and editable in any text editor.
- **Jira-Inspired Vault Switcher:** Switch between recently opened vaults from the top bar using clean, borderless selectable rows that display the folder basename, disambiguate duplicate names with muted ancestor path hints, and reveal full paths on hover.
- **Live Change Detection:** StudyFrog polls the vault directory and incrementally reindexes only modified or renamed files while preserving item identities via 26-character ULIDs.

### Item Kinds
- **Stacks (`kind: stack`):** Folders containing a `_stack.md` file. Stacks organize items into courses, modules, or topics, and can be nested arbitrarily into sub-stacks.
- **Flashcards (`kind: card`):** Two-sided prompt/answer cards separated by `???`, with optional reversible direction (`reversible: true`).
- **Questions (`kind: question`):** Free-response prompts evaluated against a reference answer, optional accepted synonyms (`accepted`), a grading rubric (`rubric`), and a minimum passing score (`min_score`).
- **Notes (`kind: note`):** Rich long-form Markdown documents with optional title-rehearsal support (`rehearse_title: true`).
- **Diary Entries (`kind: diary`):** Structured learning reflections with 1–5 self-assessment scales linked to rehearsal runs, stacks, items, and exams.

### Stack Hierarchy & Cross-Stack References
- **Flexible Stack Hierarchy:** Create standalone top-level stacks or nested sub-stacks. Move any stack under a parent stack or promote a sub-stack to a standalone top-level stack at any time from the stack page (`More actions → Move stack…`) or the stack's `Properties` tab while keeping all item schedules intact.
- **Multi-Stack Membership (`referenced_by`):** An item lives physically in its primary stack folder but can be referenced by any number of additional stacks without duplicating the file.
- **Independent Memory per Stack:** Each stack maintains its own independent FSRS-6 or SM-2 memory state, retention target, familiarity score, and review history for every item it holds or references.

---

## 2. Browsing: Folders View, Items View & Stack Workspace

### Obsidian-Style Folders View
- **Interactive File Tree:** Navigate the vault's directory hierarchy with expand/collapse controls, kind badges, sort options, inline creation (`+Note`, `+Folder`), `F2` rename with automatic Markdown link and image path rewriting, drag-and-drop moving, and undoable deletion.
- **Stack Conversion:** Turn any normal folder into a stack (or unmark a stack back into a normal folder) with a confirmation dialog and one-click undo.

### Structured Items Library & Query Language
- **Sortable, Customizable Table:** Browse all items in the vault, the Inbox, or a specific stack with toggleable columns (`Title`, `Type`, `Status`, `Priority`, `Difficulty`, `Familiarity`, `Due`, `Reviews`, `Stack`, `Also in`, `Subjects`, `Tags`, `Path`).
- **Standard & Query Search:** Filter by text, kind, status, priority, difficulty, or tags, or switch to the structured query language (`kind:card status:learning tag:exam priority:high ORDER BY due ASC`) with live autocomplete.
- **Saved Filters & Bulk Operations:** Save frequent queries as named filters. Select multiple items to bulk-update status, priority, difficulty, tags, primary stack, or stack references, export to PDF, or delete with full undo (`Undo last change`).

### Nine-Tab Stack Workspace
Opening a stack presents a dedicated workspace with nine specialized tabs (remembered per stack):
1. **Summary:** 7-day activity counters, status/priority/type breakdowns, recent item activity, and sub-stack progress bars.
2. **List:** Searchable, filterable item table scoped to the stack, due-date controls, child stack links, and attached PDFs.
3. **Backlog:** Drag-and-drop prioritization and status triage for items in the stack.
4. **Roadmap:** Interactive timeline of stack deadlines, exam dates, and scheduled milestones with zoom controls.
5. **Board:** Status-column Kanban view (`New`, `Learning`, `Review`, `Mastered`, `Paused`) for dragging items across learning stages.
6. **Calendar:** Stack-scoped calendar showing planned study sessions, deadlines, and scheduled reviews.
7. **Reports:** Stack-scoped retention, latency, forecast, and familiarity analytics.
8. **Sub-stacks:** Hierarchical tree of nested sub-stacks with completion and familiarity roll-ups.
9. **Goals:** Editable Markdown learning goals and syllabus checklist stored directly in the stack.

---

## 3. Rich Markdown Editor & Notebook Details View

### Distraction-Free & Split Markdown Editor
- **Write, Preview, and Split Modes:** Edit notes, cards, questions, and stacks with live side-by-side CommonMark + extension preview (`Write`, `Preview`, `Split`).
- **Rich Formatting & Extensions:** Supports pipe tables with column alignment, fenced code blocks with syntax highlighting, `[[Wikilinks]]` with `#heading` and `|label` syntax, underline (`<u>`), palette text colors (`<span style="color:...">`), `[TOC]` table of contents generation, smart list continuation on `Enter`, and customizable text macros.
- **Conflict-Safe Auto-Save:** Automatically saves changes as you type, detects external file edits via content hashing (`Keep mine` / `Reload from disk`), and provides one-click `Revert` and `Delete item…` actions with undo support.

### Four-Tab Details Notebook
The right-hand column of the item editor organizes metadata into a four-tab notebook:
1. **Properties:** Item kind badge, comma-separated tags, subjects and teachers, inline priority and difficulty chips, status selector, parent stack selector (for stacks), PDF source link, and creation/update timestamps.
2. **Schedule:** Current scheduler state (stability, difficulty, interval, due date, review count) and direct due-date / deadline picker.
3. **Relations:** Primary stack and cross-stack reference manager (`Add to stack…` / `Remove`), outbound `[[Wikilinks]]`, and inbound backlinks (`Linked from`).
4. **Learning:** Familiarity curve over time, per-stack familiarity and schedule breakdown, and review history.

---

## 4. Spaced Repetition & Rehearsal Engine

### Dual Scheduling Algorithms (FSRS-6 & SM-2)
- **Per-Stack Scheduler Configuration:** Choose between **FSRS-6** (modern memory-stability and retrievability model with configurable desired retention, e.g. `0.90`) and classic **SM-2** on a per-stack or vault-wide basis.
- **Familiarity Model (`0.0` – `1.0`):** Combines review accuracy, recency, latency, difficulty marks, and exam proximity into a continuous familiarity score that highlights items needing attention and tempers scheduling intervals.

### Flexible Rehearsal Runs
- **Run Modes:**
  - **Flip cards:** Read the prompt, reveal the answer (`Space`), and rate recall (`1` Again, `2` Hard, `3` Good, `4` Easy).
  - **Write answers:** Type your answer before revealing the reference; StudyFrog compares your text offline and suggests a rating.
- **Queue Orders & Presets:** Due first, Lowest recall / familiarity, Newest, Oldest, Hardest, Sequential, or Shuffled, with practice-only drill presets that rehearse without altering the schedule.
- **Pacing & Timing:** Free run, Countdown timer, or Count-up stopwatch, with pause/resume (`P`), difficulty marking (`E` Easy, `M` Medium, `D` Difficult), and reading-speed baseline tracking.
- **Offline Semantic Answer Comparison:** Evaluates typed answers using an optional local embedding model (`all-MiniLM-L6-v2` via `studyfrog-candle`) or fast token/word-overlap similarity when no model is installed.
- **Post-Run Review & Rerate:** Scrollable run summary showing rating distributions, median answer latency, per-item review cards with `Suggested rating` and reading pace (`words in s`), one-click rerating, and an inline learning diary prompt.

---

## 5. Planning, Calendar, Kanban Board & Diary

### Calendar & Review Forecasting
- **Month & Week Views:** Displays planned study sessions, stack deadlines, exam dates, and projected spaced-repetition review loads.
- **Interactive Rescheduling:** Drag scheduled reviews or study sessions to another day, configure recurring study sessions, and set reminder lead times.

### Scrum / Kanban Board
- **Customizable Columns & WIP Limits:** Manage study tasks and cards across board columns with Work-In-Progress (WIP) limits, story point estimates, stack links, drag-and-drop ordering, and full keyboard accessibility.

### Learning Diary & Calibration
- **Structured Self-Assessment:** Log daily or post-rehearsal diary entries using 13 built-in 1–5 scales (such as *Focus & Attention*, *Energy Level*, *Satisfaction & Retention*, *Understanding*, *Confidence*) plus custom user-defined scales.
- **Calibration Analytics:** The Reports screen compares subjective diary ratings against objective review accuracy and familiarity to surface overconfidence or underconfidence trends.

---

## 6. Analytics, Reports, Graph & Achievements

- **Reports & Performance Explorer:** Inspect retention rates, answer-time distributions, 30-day review forecasts, lowest-familiarity items (`Needs attention`), per-stack mastery, and self-assessment calibration.
- **Interactive Link Graph:** Pan, zoom, filter, and click through the vault's `[[Wikilink]]` network (`5,000` nodes rendered in ~4 ms per frame with zero per-frame heap allocations).
- **Achievements:** Unlock global and per-stack milestones for streaks, review volume, mastery percentages, and study consistency, customizable via `achievements.toml`.

---

## 7. PDF Reader, Annotations & Markdown-to-PDF Export

### Built-In PDF Workspace
- **Read & Search:** View PDF documents inside the vault with page thumbnails, zoom, full-text search across all indexed vault PDFs, and page rearranging.
- **Highlights, Notes & Flashcard Extraction:** Highlight passages, attach margin notes, extract regions/images, and turn PDF selections directly into linked StudyFrog notes or flashcards (`source: file.pdf#page=N`).

### Native Markdown-to-PDF Export
- **Pure Rust Layout & Writer:** Export any single item, multi-item library selection, or entire stack (including sub-stacks) to a formatted PDF document with configurable page size (`A4`, `Letter`), margins, font size, title page, clickable table of contents, syntax-highlighted code blocks, tables, embedded images, and live preview.

---

## 8. Import, Export, AI Assistant & Prompt Library

### Versatile Import & Export
- **CSV & TSV with Smart Recovery:** Auto-detects delimiters (`,`, `\t`, `;`, `|`), encodings (UTF-8, Latin-1), headers, and column mappings. Automatically reconstructs unquoted commas and unquoted multiline fields in AI-generated CSV exports and unescapes `\n` / `<br>` line breaks.
- **Anki (`.apkg`) & Obsidian Import:** Import decks, notes, media attachments, and review histories from Anki packages and Obsidian vaults, with automatic discovery of files in common folders.
- **Structured Export:** Export stacks or selections to **Markdown**, **CSV**, **JSON**, **TOML**, or **RON** (with optional scheduling and review history).

### Built-In Prompt Library & Optional AI Assistant
- **Ready-Made Prompt Library:** Six built-in, importer-verified prompts (`Flashcards`, `Questions`, `Whole Stack`, `Notes`, `Summary`, and `Critique`) that instruct any external LLM (Gemini, ChatGPT, Claude) or connected assistant to produce valid StudyFrog CSV batches from attached study material, with one-click clipboard copy and Markdown export.
- **Human-in-the-Loop Assistant:** Optional support for local (`Ollama`) or remote (`OpenAI`-compatible, `Anthropic`) LLM endpoints. The assistant never modifies your vault directly; all proposed notes, cards, and edits are staged for explicit preview and approval.

---

## 9. Appearance, Accessibility & Guardrails

- **Customizable Appearance:** Dark, Light, or System theme, UI scale, text size, layout density, accent color palette, high-contrast mode, and reduced motion, all applied live.
- **Keyboard & Screen-Reader Accessible:** Every interactive control carries an AccessKit role and accessible name (verified in CI by headless `egui_kittest` audits), with keyboard navigation (`Alt+Left` / `Alt+Right` history, `F1` in-app Help & tutorial, keyboard rehearsal).
- **Automated Backups & Profiles:** Create and restore timestamped `.tar.gz` vault backups and switch between user profiles.
