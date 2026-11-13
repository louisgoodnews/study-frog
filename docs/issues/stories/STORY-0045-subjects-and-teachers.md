---
id: STORY-0045
type: story
title: "Subjects and teachers"
status: done
epic: EPIC-021
parents: [EPIC-021]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "A teacher has subjects; a stack has subjects and its items inherit them"
---

## Context
The owner reviewed the app again and asked for a knowledge workspace: an item page, images, subjects, reports, an AI agent and more.

## Description
Subjects and teachers.

## Acceptance Criteria
- [x] A teacher has subjects; a stack has subjects and its items inherit them

## Design Notes
Teachers and subjects live in subjects.toml; a stack names its subjects in its front matter; items inherit. Stored in _stack.toml and the index.

## Dependencies
None.

## Open Questions
None.
