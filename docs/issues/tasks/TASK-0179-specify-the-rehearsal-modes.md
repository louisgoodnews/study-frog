---
id: TASK-0179
type: task
title: "Specify the rehearsal modes"
status: done
epic: EPIC-023
parents: [STORY-0070]
estimate: 2
labels: [m10, docs, decision]
acceptance_criteria:
  - "docs/specs/rehearsal-run.md describes the modes, what happens at zero, the pause rule, the warnings and the stored fields"
  - "The behaviour at the end of a countdown (the open item counts as not answered, with no rating) is written down"
---

## Context
The owner asked on 2026-11-02 for the 0.3.0 release to focus on usability and insight: a fuller notification center, the vault in the bar, a redesigned library and stack view (Jira as the reference), a familiarity score, tabbed settings, item statuses and calendar scheduling that respects them.

## Description
Extend the rehearsal run spec before building.

## Acceptance Criteria
- [x] docs/specs/rehearsal-run.md describes the modes, what happens at zero, the pause rule, the warnings and the stored fields
- [x] The behaviour at the end of a countdown (the open item counts as not answered, with no rating) is written down

## Design Notes
Written 2026-11-02 in `docs/specs/rehearsal-run.md` (section 'Pacing'). The open item at the end of a countdown counts as not answered, as the owner decided. Proposals not yet confirmed: the 1 to 180 minute limits, no extension of a running countdown, warnings at 60 and 10 seconds.

## Dependencies
None.

## Open Questions
None.
