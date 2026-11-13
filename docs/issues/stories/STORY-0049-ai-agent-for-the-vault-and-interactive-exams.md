---
id: STORY-0049
type: story
title: "AI agent for the vault and interactive exams"
status: done
epic: EPIC-021
parents: [EPIC-021]
estimate: 8
labels: [m8, ui]
acceptance_criteria:
  - "A local or remote model can read and maintain the vault like an LLM wiki and examine the user in a session"
---

## Context
The owner reviewed the app again and asked for a knowledge workspace: an item page, images, subjects, reports, an AI agent and more.

## Description
AI agent for the vault and interactive exams.

## Acceptance Criteria
- [x] A local or remote model can read and maintain the vault like an LLM wiki and examine the user in a session

## Design Notes
Crates studyfrog-agent (pure protocols, prompts, checks) and studyfrog-net (the only one linking ureq); service::agent and agent_vault; the Assistant screen. Implements ADR 0009. Decided in ADR 0009 (accepted).

## Dependencies
None.

## Open Questions
None.
