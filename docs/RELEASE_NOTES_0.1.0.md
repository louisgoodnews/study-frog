# StudyFrog 0.1.0

A Linux desktop app for learning from your own Markdown notes: link them, turn them into flashcards, rehearse
them with spaced repetition, plan your studying, and see how it is going. Everything stays in a folder of plain
files plus a rebuildable index. Nothing leaves your machine unless you connect a remote AI model and allow it.

## What it does
- **Vault.** Open a folder of Markdown files (notes, cards, questions, stacks; stacks can sit inside stacks).
  The item page edits any of them with a Markdown toolbar and live preview, conflict-safe saving, `[[wikilinks]]`
  with "Linked from", pictures (`assets/`), priority, difficulty, subjects and tags. The files are the truth; the
  SQLite index is rebuilt from them.
- **Home.** What is due now, the streak, the stacks waiting, today's sessions, the next seven days and recent
  activity, with one button to start today's rehearsal.
- **Rehearsal.** A three-step setup (what, how, review), then flip cards or type answers on a card with a progress
  bar. FSRS-6 or SM-2 per stack; priority orders the queue; shuffle works with any order. Mark items easy, medium
  or difficult. A review after each run lists every item with previous rating, your mark, your rating, the answer
  match and a suggested rating from your reading speed (measure it in Settings).
- **Planning.** A calendar with study sessions, tasks, appointments, exams and deadlines, repeats and reminders; a
  stack can have a due date (a stack past its date is no longer suggested). A kanban board with WIP limits and
  drag and drop; new items get a board card.
- **Progress.** Achievements, retention by interval, time to answer, a 30-day forecast, and a performance explorer
  from the whole vault down to one item, by period, with a subject breakdown.
- **Assistant.** Connect a local model (Ollama and similar) or a remote one (OpenAI-compatible, Anthropic). It can
  read stacks you choose and propose notes, cards, questions, edits and an index, which you accept one by one, and
  it can examine you, recording the verdicts as reviews only when you say so. See `docs/specs/agent.md`.
- **Backups and profiles.** One `.tar` with files, database (reviews, board, calendar), settings and optionally
  keys; restore into a new folder. Several profiles with their own settings and keys.
- **Typed answers** are compared offline: word matching built in, or a small language model (MiniLM) in
  `~/.local/share/studyfrog/models/all-MiniLM-L6-v2/`.
- **Extensions.** Add importers and card types by writing a small crate; see `docs/EXTENDING.md`.

## Requirements
Linux with Wayland or X11, OpenGL, and for the folder and file dialogs an XDG desktop portal. The AppImage
contains only the application.

## Things to know
- **Checked by hand on a desktop:** the AppImage starts and opens a vault; the home, calendar and rehearsal
  screens were looked at in a window. **Checked only by automated tests, not by hand:** the item page, Markdown
  toolbar, date picker, assistant, backup and profile menus, board drag and drop, native dialogs, IME input and
  screen readers.
- **The assistant** has never talked to a real provider; the three wire formats follow the providers'
  documentation and are tested against a local fake server. Remote models receive the text of the items you let
  them read. API keys are stored unencrypted in a file only you can read.
- Typed-answer matching measures similarity, not truth: a wrong answer that differs by one decisive word usually
  scores as correct. Treat the match as a hint; you rate.
- The forecast was checked against a simulated history (weekly totals within about 10 %), not against months of
  real use.
- Extensions are compiled in; there is no plugin loading.
- Formats may still change before 1.0 (schema migrations are provided; the database is at schema 10).
- The CI workflow in `.github/workflows/ci.yml` has never run.

## Files
`StudyFrog-0.1.0-x86_64.AppImage` and its SHA-256 (`.sha256`). `chmod +x` it and run it.
