---
id: TASK-0051
type: task
title: Model management
status: done
epic: EPIC-012
parents: [STORY-0023]
estimate: 3
labels: [m6, slm]
acceptance_criteria:
  - "Models live in the app data directory; missing or corrupt models are reported with how to fix it"
  - "The runtime is chosen automatically: the semantic one when the model is installed, else word matching"
---

## Context
Nothing is bundled.

## Description
Model management.

## Acceptance Criteria
- [x] Models live in the app data directory; missing or corrupt models are reported with how to fix it
- [x] The settings choose lexical or a named model

## Design Notes
`slm::models`: `models_dir` (XDG data dir), `missing_files`, `install_hint`, `open_runtime` (semantic when installed and loadable, else lexical, with the reason). The runtime is opened on first use by the UI worker. The settings do not yet let the user force word matching; "auto" is the only mode.

## Dependencies
TASK-0050.

## Open Questions
None.
