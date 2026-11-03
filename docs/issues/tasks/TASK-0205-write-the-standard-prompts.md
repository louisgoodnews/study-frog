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
  - "Each prompt says what is attached, what to produce and the exact layout of the answer, so that StudyFrog can import it"
  - "A README lists the prompts, their placeholders and the formats they ask for"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports.

## Description
Prompts that work in any chatbot and with StudyFrog's assistant, whose answers can be pasted or imported.

## Acceptance Criteria
- [ ] docs/prompts/ holds one Markdown file each for: create flashcards from the attached file, create questions and answers, create a stack, create notes, analyze and summarize, analyze and critique
- [ ] Each prompt says what is attached, what to produce and the exact layout of the answer, so that StudyFrog can import it
- [ ] A README lists the prompts, their placeholders and the formats they ask for

## Design Notes
The answer layouts must be formats the importers read (CSV with the header, the vault's Markdown cards with the `???` split, the stack folder layout). Compiled into the binary like the help pages, with a test that checks the layouts against the importers. English first.

## Dependencies
TASK-0209 for the layouts.

## Open Questions
Which languages (English and German?), and should a prompt know the user's stack names?
