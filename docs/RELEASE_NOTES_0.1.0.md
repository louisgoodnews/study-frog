# StudyFrog 0.1.0 (draft)

A Linux desktop app for learning from your own Markdown notes: link them, turn them into flashcards, rehearse
them with spaced repetition, plan study sessions, and see how it is going. Everything stays in a folder of
plain files plus a rebuildable index; nothing leaves your machine.

## What it does
- **Vault.** Open a folder of Markdown files (notes, cards, questions, stacks). Edit with a live preview and
  conflict-safe saving; follow `[[wikilinks]]`; see the link graph. The files are the truth; the SQLite index is
  rebuilt from them.
- **Rehearsal.** Run through due items (FSRS-6 or SM-2 per stack), or everything in order, shuffled, hardest
  first, or not seen for longest; shuffle works with any of them. Flip cards or type answers. Time to answer is
  recorded without paused time. Mark items easy, medium or difficult as you go.
- **Review after a run.** Previous rating, your mark and rating, how your typed answer matched, and a suggested
  rating from your reading speed and your mark, with a one-click way to apply it. Measure your reading speed in
  Settings.
- **Typed answers** are compared offline: word matching built in, or a small language model (MiniLM) if you
  place it in `~/.local/share/studyfrog/models/all-MiniLM-L6-v2/`.
- **Planning.** A calendar with repeating study sessions, and a kanban board with work-in-progress limits.
- **Progress.** Achievements (your own in `achievements.toml`, or the built-in set); retention by interval,
  time to answer, and a 30-day review forecast.
- **Extensions.** Add importers and card types by writing a small crate; a tab-separated importer and a cloze
  card type are included. See `docs/EXTENDING.md`.

## Requirements
Linux with Wayland or X11, OpenGL, and for the folder and file dialogs an XDG desktop portal. The AppImage
contains only the application.

## Things to know
- Typed-answer matching measures similarity, not truth: a wrong answer that differs by one decisive word
  usually scores as correct. Treat the match as a hint; you rate.
- The forecast has been checked against a simulated history (weekly totals within about 10 %), not yet against
  months of real use.
- Extensions are compiled in; there is no plugin loading.
- The vault index and settings formats may still change before 1.0 (schema migrations are provided).
- Verified in this repository: the gate, headless UI tests, benchmarks. Not verified: the native dialogs on a
  real desktop, IME input, real screen-reader behaviour, and the CI workflow (it has never run).
