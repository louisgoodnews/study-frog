---
id: TASK-0247
type: task
title: "scripts.toml, cron and trigger matching"
status: done
epic: EPIC-028
parents: [STORY-0132]
estimate: 3
labels: [m12, service]
acceptance_criteria:
  - "Config parsing with per-entry errors, the cron parser and matcher, due jobs per minute and jobs per event; pure, unit tested with an injected clock"
---
## Context
Part of STORY-0132.

## Description
See the story and ADR 0013.

## Acceptance Criteria
- [x] Config parsing with per-entry errors, the cron parser and matcher, due jobs per minute and jobs per event; pure, unit tested with an injected clock

## Design Notes
See the story.

## Dependencies
STORY-0132.

## Open Questions
None.
