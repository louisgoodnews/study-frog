---
id: STORY-0251
type: story
title: "Highlight active Rehearse setup preset and streamline run item-limit selection"
status: backlog
epic: EPIC-073
parents: ['EPIC-073']
estimate: 2
labels: [m14, ui, rehearse]
acceptance_criteria:
  - 'The active Rehearse setup preset (Spaced repetition, Everything in order, Everything shuffled, Hardest first, Not seen for longest) is visually highlighted in Step 2 (How) whenever the current settings match that preset'
  - 'Setting a maximum number of items for a run is clear, ergonomic (quick preset chips plus custom numeric input with validation), and reliably caps the preview count and actual run queue'
  - 'Headless UI tests verify preset highlighting and run item-limit enforcement'
---

## Context
Reported by the user with a screenshot of **Rehearse -> Step 2 (`How do you want to rehearse?`)**:
1. The top preset buttons (`Spaced repetition`, `Everything, in order`, `Everything, shuffled`, `Hardest first`, `Not seen for longest`) use plain `ui.button(...)` rather than `selectable_label`, so none of them is highlighted to indicate the currently selected mode/preset.
2. Users want an intuitive, reliable way to cap the number of items included in a run (`Maximum number of items` with quick-pick chips and verified enforcement in the preview and queue).

## Description
Upgrade `crates/studyfrog-ui/src/view/setup_how.rs` and `crates/studyfrog-ui/src/setup/` so the active preset is highlighted and run item limits are easy to set and strictly enforced (`TASK-0315`).

## Acceptance Criteria
- [ ] The active Rehearse setup preset (`Spaced repetition`, `Everything, in order`, `Everything, shuffled`, `Hardest first`, `Not seen for longest`) is visually highlighted in Step 2 (`How`) whenever the current settings match that preset
- [ ] Setting a maximum number of items for a run is clear, ergonomic (quick preset chips plus custom numeric input with validation), and reliably caps the preview count and actual run queue
- [ ] Headless UI tests verify preset highlighting and run item-limit enforcement

## Design Notes
- Child task: `TASK-0315`

## Dependencies
- `EPIC-073`

## Open Questions
None.
