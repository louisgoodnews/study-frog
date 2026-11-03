---
id: TASK-0210
type: task
title: Decide how a stack points to its contents (ADR and spec)
status: backlog
epic: EPIC-026
parents: [STORY-0079]
estimate: 3
labels: [m11, docs, decision]
acceptance_criteria:
  - "An ADR and an update of docs/specs/markdown-schema.md state the stack file's format, how it names its contents (ids, paths or wikilinks), whether an item may be in several stacks, the order of the contents, how stacks hold stacks, and where the stack's settings live"
  - "It states what a folder is (only organization), what happens to existing vaults and how they are migrated, and the effect on import, export and the Obsidian and Anki importers"
  - "The owner has accepted it"
---

## Context
The owner tested the 0.3.1 build on 2026-11-03 and listed what to do in 0.4.0: a better Markdown editor, saving and creating, organizing the vault, prompts for AI agents and clearer imports. The owner answered the open questions the same day (recorded below).

## Description
Write the decision down before changing the model.

## Acceptance Criteria
- [ ] An ADR and an update of docs/specs/markdown-schema.md state the stack file's format, how it names its contents (ids, paths or wikilinks), whether an item may be in several stacks, the order of the contents, how stacks hold stacks, and where the stack's settings live
- [ ] It states what a folder is (only organization), what happens to existing vaults and how they are migrated, and the effect on import, export and the Obsidian and Anki importers
- [ ] The owner has accepted it

## Design Notes
Owner's words (2026-11-03): folders are folders and not stacks; a stack is a file that points to its contents; a folder contains contents and is mostly organizational, whereas the stack also employs logic. Questions the record must answer: a list of ids in the front matter, wikilinks in the body (readable in other tools, links survive moves) or a saved query; ordering and manual order; one item in several stacks (and then the schedule, the status and the roll-ups); sub-stacks as stacks that point to stacks; the item's 'parent stack' of the Create dialog; `_stack.toml` becomes the stack file's own settings or stays beside it; how an Obsidian vault and an Anki deck map; what the vault looks like for someone who opens it without StudyFrog.

## Dependencies
None.

## Open Questions
Can an item belong to several stacks?
