# Vision

## Product

StudyFrog is a Linux-native desktop application that turns a folder of Markdown into a
complete learning system: a linked note graph (Obsidian-like), spaced-repetition flashcards
(Anki-like), timed rehearsal with latency capture, a planning calendar, a kanban/scrum board,
achievements, and local small-language-model (SLM) grading of free-text answers.

Everything the user authors is plain Markdown with front-matter. The app's databases are
derived indexes and history, never the only copy of the user's knowledge.

## Target user

A self-directed learner (student, researcher, certification candidate, language learner) who
already keeps notes in Markdown, wants evidence-based review scheduling, and values owning
their data and running offline on their own hardware.

## Principles

1. **Your files, your data.** Content is readable and editable without StudyFrog.
2. **Local first.** No account, no network requirement, SLM runs on-device.
3. **Evidence-based.** Scheduling and practice modes follow published learning science
   (see [learning-science](research/learning-science.md)).
4. **Measured.** Latency and outcomes are first-class data, feeding forecasts.
5. **Fast.** Startup, rehearsal loop, graph rendering and SLM scoring are benchmarked.

## Non-goals

- Cloud sync, accounts, or a hosted service (a user may sync the vault folder themselves).
- Mobile or Windows/macOS builds in the initial roadmap (Linux first).
- Anki `.apkg` binary compatibility as a core feature (import/export may come later).
- Cloud LLM calls. The SLM is local only.
- A general plugin marketplace; M8 defines internal boundaries only.
- Real-time collaboration.
