---
id: STORY-0249
type: story
title: "Verify and preserve Question items and answer types across all import formats"
status: backlog
epic: EPIC-072
parents: ['EPIC-072']
estimate: 3
labels: [m14, import, service, qa]
acceptance_criteria:
  - 'Question items and their answer types (open, single, multiple, case, cloze) are audited across all supported import formats'
  - 'Front-matter metadata (including answer_type and choice options) survives import and export -> import round-trips without data loss'
  - 'Automated service tests verify Question and answer-type import fidelity'
---

## Context
With choice questions (`EPIC-072`) and other question answer types (`open`, `single`, `multiple`, `case`, `cloze`) supported in StudyFrog, the universal import/export pipeline (`xfer_*`) needs verification to ensure questions and their answer types import and round-trip cleanly without silent data loss.

## Description
Audit all supported importers for `Question` items and answer types, fix any preservation gaps in `studyfrog-service`, and cover them with integration tests (`TASK-0312`).

## Acceptance Criteria
- [ ] Question items and their answer types (`open`, `single`, `multiple`, `case`, `cloze`) are audited across all supported import formats
- [ ] Front-matter metadata (including `answer_type` and choice options) survives import and export -> import round-trips without data loss
- [ ] Automated service tests verify Question and answer-type import fidelity

## Design Notes
- Child task: `TASK-0312`

## Dependencies
- `EPIC-072`

## Open Questions
None.
