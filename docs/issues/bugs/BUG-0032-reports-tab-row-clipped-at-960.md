---
id: BUG-0032
type: bug
title: "At 960 by 640 the Reports tab row is cut off at the right edge"
status: done
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
- [x] At 960 by 640 every Reports tab (Performance to Workload) can be reached: the tab row wraps or scrolls

## Design Notes
- Fixed on 2026-11-23. Cause: `view/reports.rs` put the nine tab buttons in one non-wrapping row. Fix: the row is `horizontal_wrapped`. Before: `/tmp/claude-0/layout/before-reports-960.png` (Habits cut to "Hab", Leeches and Workload off-screen). After: `/tmp/claude-0/layout/after3-home-960.png` (Reports at 960 by 640: all nine tabs visible, the last three on a second line). Test: `studyfrog-ui/tests/window_fit_layout.rs::every_reports_tab_is_inside_the_window_at_960_by_640`.
- None.

## Dependencies
- STORY-0178

## Open Questions
- None.
