---
id: TASK-0207
type: task
title: Run a prompt with the connected assistant and an attached file
status: backlog
epic: EPIC-026
parents: [STORY-0077]
estimate: 5
labels: [m11, ui, agent]
acceptance_criteria:
  - "With a connection chosen the user picks a prompt and a file, sees what will be sent, and sends it only after pressing Send, as for PDF pages"
  - "The answer is shown and, where it is in an import layout, can be imported through the preview"
  - "If the import finds problems, a button composes a message with the problems for the assistant and sends it after Send, so the AI can fix its file"
  - "Nothing is sent to a remote provider that is not allowed, as today"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
The same prompts through StudyFrog's own assistant, including the fix-it round trip.

## Acceptance Criteria
- [ ] With a connection chosen the user picks a prompt and a file, sees what will be sent, and sends it only after pressing Send, as for PDF pages
- [ ] The answer is shown and, where it is in an import layout, can be imported through the preview
- [ ] If the import finds problems, a button composes a message with the problems for the assistant and sends it after Send, so the AI can fix its file
- [ ] Nothing is sent to a remote provider that is not allowed, as today

## Design Notes
Extend the assistant's wiki flow (`agent_run`) with a prompt and an attachment; the attachment's text is read locally (Markdown, text, source and data files, PDF pages) and counted against the context budget; the reply goes through `parse_proposal` or the import preview depending on the prompt.

## Dependencies
TASK-0205, TASK-0206 and TASK-0208.

## Open Questions
None.
