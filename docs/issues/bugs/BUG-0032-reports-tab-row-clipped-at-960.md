---
id: BUG-0032
type: bug
title: "At 960 by 640 the Reports tab row is cut off at the right edge"
status: backlog
estimate: 2
labels: [m14, ui, report]
epic: EPIC-060
parents: ['EPIC-060']
acceptance_criteria:
  - "At 960 by 640 every Reports tab (Performance to Workload) can be reached: the tab row wraps or scrolls"
---

## Context
Found while auditing STORY-0178 (2026-11-22, display :92). At 960 by 640 the Reports tab row runs past the window: 'Habits' is cut to 'Hab' and 'Leeches' and 'Workload' are not visible. Screenshot `/tmp/claude-0/audit/g2/05-cognitive-960.png`. The Cognitive profile tab is the one this epic adds.

## Description
Let the tab row wrap onto a second line or scroll sideways at narrow widths. Reproduction: resize the window to 960 by 640, open Reports.

## Acceptance Criteria
- [ ] At 960 by 640 every Reports tab (Performance to Workload) can be reached: the tab row wraps or scrolls

## Design Notes
- None.

## Dependencies
- STORY-0178

## Open Questions
- None.
