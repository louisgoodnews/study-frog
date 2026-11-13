---
id: STORY-0051
type: story
title: "User profiles"
status: done
epic: EPIC-021
parents: [EPIC-021]
estimate: 5
labels: [m8, ui]
acceptance_criteria:
  - "More than one profile can exist, each with its own settings and recent vaults"
  - "The profile menu switches between them"
---

## Context
The owner asked for this after reviewing the app on 2026-10-30.

## Description
User profiles.

## Acceptance Criteria
- [x] More than one profile can exist, each with its own settings and recent vaults
- [x] The profile menu switches between them

## Design Notes
profiles.toml in the config folder; the Default profile keeps app.toml in the config folder itself, the others live in profiles/<slug>/ with their own settings and keys. Per-profile directory under the config dir.

## Dependencies
None.

## Open Questions
None.
