---
id: STORY-0044
type: story
title: Images in notes, cards and questions
status: done
epic: EPIC-021
parents: [EPIC-021]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "Images are copied into the vault, referenced from Markdown and shown in the editor, preview and rehearsal"
---

## Context
The owner reviewed the app again and asked for a knowledge workspace: an item page, images, subjects, reports, an AI agent and more.

## Description
Images in notes, cards and questions.

## Acceptance Criteria
- [x] Images are copied into the vault, referenced from Markdown and shown in the editor, preview and rehearsal

## Design Notes
Pictures live in the vault's assets folder (png, jpg, gif up to 20 MB), are referenced as ![alt](assets/name.png) and drawn by view/image.rs, which loads them itself because egui's image widget prints a warning sign the bundled fonts cannot draw. Needs an image loader (egui_extras).

## Dependencies
None.

## Open Questions
None.
