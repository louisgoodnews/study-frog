---
id: STORY-0067
type: story
title: Importers for Anki and Obsidian
status: done
epic: EPIC-023
parents: [EPIC-023]
estimate: 8
labels: [m10, app, md, ext]
acceptance_criteria:
  - "An Anki export (.apkg) imports its decks as stacks and its notes as cards, with tags, pictures and review history, and the schedule of each card is adapted from that history"
  - "An Obsidian vault imports its Markdown files as notes in stacks that follow the folders, with links, tags and pictures kept"
  - "Both go through the preview and duplicate handling of STORY-0065"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Bring a user's existing material into StudyFrog. These are the first two format adapters; the goal is to support as many formats as possible.

## Acceptance Criteria
- [x] An Anki export (.apkg) imports its decks as stacks and its notes as cards, with tags, pictures and review history, and the schedule of each card is adapted from that history
- [x] An Obsidian vault imports its Markdown files as notes in stacks that follow the folders, with links, tags and pictures kept
- [x] Both go through the preview and duplicate handling of STORY-0065

## Design Notes
Each format is an adapter in the extension registry (ADR 0008) with a decoder from the file to the interchange model of STORY-0065 and nothing else, so a new format touches no core code. Anki: an .apkg is a zip with a SQLite collection and a media map; read decks, note types (basic and cloze), fields, tags, media and the review log. The owner decided on 2026-11-02 that the history is imported and the schedule adapted: each card's reviews are replayed through the stack's scheduler (FSRS or SM-2) so stability, difficulty and the next due date follow from what the user actually did, not from Anki's own numbers; a checkbox in the import preview leaves the history out, and then cards start as new. Obsidian: a folder of Markdown files, usually with YAML front-matter, `[[wikilinks]]`, `#tags` and an attachments folder; folders become stacks, links and tags are kept, and Obsidian-only syntax that StudyFrog does not know (callouts, embeds, dataview) is kept as plain text and counted in the preview. Neither source is changed. Hostile files must not crash or hang the app (the PDF safety work is the model).

Tasks: TASK-0167, TASK-0168, TASK-0169.

## Dependencies
STORY-0065 (the import engine and the interchange model).

## Open Questions
Answered by the owner on 2026-11-02: the review history is imported and the schedule adapted (replay through the scheduler). Anki's media and note-type features that StudyFrog lacks are listed in the preview rather than dropped silently.
