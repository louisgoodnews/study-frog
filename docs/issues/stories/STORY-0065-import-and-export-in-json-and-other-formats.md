---
id: STORY-0065
type: story
title: Import and export in JSON and other formats
status: backlog
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, app, md, ui]
acceptance_criteria:
  - "The library or a chosen stack can be exported as JSON, CSV and a folder of Markdown files, and a JSON export can be imported again with nothing lost"
  - "Import shows a preview with counts, duplicates and errors before anything is written"
  - "Anki and Obsidian are imported first (STORY-0067) and further formats are added through the same registry; the long-term goal is to support as many formats as possible"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Move data in and out of StudyFrog in open formats, for sharing a stack, for other tools and for keeping a copy outside the app.

## Acceptance Criteria
- [ ] The library or a chosen stack can be exported as JSON, CSV and a folder of Markdown files, and a JSON export can be imported again with nothing lost
- [ ] Import shows a preview with counts, duplicates and errors before anything is written
- [ ] Anki and Obsidian are imported first (STORY-0067) and further formats are added through the same registry; the long-term goal is to support as many formats as possible

## Design Notes
This is interchange, not backup: STORY-0050 backs up the whole vault for restoring it; export writes a chosen part in a documented format. JSON carries a format version and holds stacks, notes, cards, questions, tags, links, status, priority and difficulty, due dates and the review history and schedule only in the advanced export mode. CSV and TSV carry cards and questions (front, back, tags, stack). A Markdown export is the vault layout with the front-matter, so it can be opened elsewhere. Import runs a dry run first that lists what would be created, what already exists (by id, else by front and back) and what fails, then writes only after the user confirms. Importers are registered through the extension traits (ADR 0008) so a new format is a record in the registry. The existing import (`import_ui.rs`, `exec_import.rs`) becomes one of them. Pictures and PDFs travel in a zip beside the JSON.

Tasks: TASK-0156, TASK-0157, TASK-0158, TASK-0159.

## Dependencies
None; export of statuses and familiarity inputs follows STORY-0061 and STORY-0058 once they exist.

## Open Questions
Answered by the owner on 2026-11-02: Anki and Obsidian first; the goal is as many formats as possible, so every format is an adapter in the registry. Later candidates: Quizlet text, OPML for the stack tree. Answered by the owner on 2026-11-02: scheduling is left out by default, and every export opens a modal that asks for the export mode, Default (without scheduling) or Advanced (options; for now the only one is to include scheduling, that is the review history and schedule). More advanced options can be added to the modal later
