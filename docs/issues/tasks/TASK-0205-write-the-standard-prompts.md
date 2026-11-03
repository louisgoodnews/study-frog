---
id: TASK-0205
type: task
title: Write the standard prompts
status: backlog
epic: EPIC-026
parents: [STORY-0077]
estimate: 5
labels: [m11, docs, agent]
acceptance_criteria:
  - "docs/prompts/ holds one Markdown file each for: create flashcards from the attached file, create questions and answers, create a stack, create notes, analyze and summarize, analyze and critique"
  - "Each prompt fully instructs and hand-holds the AI: what is attached, what to produce, the exact layout of the answer with a column table and a worked example, how to escape commas, quotes and line breaks, the encoding, and a checklist the AI must pass before answering"
  - "Each prompt says to answer with only the file in one code block, and what to do when the attached file is not enough"
  - "The prompts cope with attached files of the kinds the owner named: python, r, rust, php, java, toml, html, json, csv, tsv, cpp and ordinary text and PDF text; for source and data files they say what to extract (concepts, functions, schemas, edge cases) instead of treating them as prose"
  - "A test feeds the example in each prompt to the importer and it is accepted"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
The owner tried a CSV that Gemini produced and the application could not read it: the AI had not been told the layout, and the owner did not know it either. The prompts must make that work the first time.

## Acceptance Criteria
- [ ] docs/prompts/ holds one Markdown file each for: create flashcards from the attached file, create questions and answers, create a stack, create notes, analyze and summarize, analyze and critique
- [ ] Each prompt fully instructs and hand-holds the AI: what is attached, what to produce, the exact layout of the answer with a column table and a worked example, how to escape commas, quotes and line breaks, the encoding, and a checklist the AI must pass before answering
- [ ] Each prompt says to answer with only the file in one code block, and what to do when the attached file is not enough
- [ ] The prompts cope with attached files of the kinds the owner named: python, r, rust, php, java, toml, html, json, csv, tsv, cpp and ordinary text and PDF text; for source and data files they say what to extract (concepts, functions, schemas, edge cases) instead of treating them as prose
- [ ] A test feeds the example in each prompt to the importer and it is accepted

## Design Notes
Owner's answers (2026-11-03): the languages are python, r, rust, php, java, toml, html, json, csv, tsv, cpp 'etc.' (the kinds of files attached, and the languages for code blocks in TASK-0214); the CSV failure was a missing format, so the prompts are the fix. The layouts come from TASK-0209 and are tested against the importers, so a prompt can never drift from what the application reads. Compiled into the binary like the help pages. English.

## Dependencies
TASK-0209 for the layouts, TASK-0208 for the CSV rules.

## Open Questions
None.
