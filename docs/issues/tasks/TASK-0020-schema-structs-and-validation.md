---
id: TASK-0020
type: task
title: Schema structs and validation
status: done
epic: EPIC-005
parents: [STORY-0009]
estimate: 3
labels: [m2, md]
acceptance_criteria:
  - "Typed front-matter structs per kind (stack, note, card, question) with defaults"
  - "Validation errors carry file and key; duplicate ids detected"
  - "Unknown keys preserved in a remainder map"
---

## Context
Spec: docs/specs/markdown-schema.md.

## Description
Schema structs and validation.

## Acceptance Criteria
- [x] Typed front-matter structs per kind (stack, note, card, question) with defaults
- [x] Validation errors carry file and key; duplicate ids detected
- [x] Unknown keys preserved in a remainder map

## Design Notes
Docs (`NoteDoc`, `CardDoc`, ...) carry no stack reference: stack membership comes from the folder (scanner). Validation lives in `build.rs`: ULID id, non-blank single-line title/tags/aliases (no control or line-break characters), RFC 3339 timestamps (years 0000-9999), `min_score` in [0,1], `srs.request_retention` in (0,1); unknown `srs` override keys are an error (never silently dropped). Duplicate ids: `validate::find_duplicate_ids`. Unknown keys are kept verbatim (`RawEntry`). ULID and RFC 3339 codecs are in-house (P10).

## Dependencies
None.

## Open Questions
Where do remainder keys live in core types (side table keyed by Uid vs field)?
