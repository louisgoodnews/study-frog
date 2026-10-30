---
id: TASK-0082
type: task
title: Reading-speed rating suggestion
status: done
epic: EPIC-014
parents: [STORY-0035]
estimate: 3
labels: [m6, rehearsal]
acceptance_criteria:
  - "Reading time is the time between reveal and rating, paused time excluded"
  - "Baseline 238 words per minute; faster than 75 % of the expected time counts as fast; backs under 8 words are ignored"
  - "A known item rated Good becomes Easy; a new item rated Good or Easy becomes Hard; low ratings are never overridden"
---

## Context
A fast read means either known well (easy) or not read because it means nothing yet (hard).

## Description
Reading-speed rating suggestion.

## Acceptance Criteria
- [x] Reading time is the time between reveal and rating, paused time excluded
- [x] Baseline 238 words per minute; faster than 75 % of the expected time counts as fast; backs under 8 words are ignored
- [x] A known item rated Good becomes Easy; a new item rated Good or Easy becomes Hard; low ratings are never overridden

## Design Notes
`studyfrog-rehearsal::reading`, pure and property-tested. Whether the item is new comes from its review history before the run. Words are counted by whitespace, so scripts without spaces are not measured well. The constants are defaults in code, not yet settings.

## Dependencies
None.

## Open Questions
None.
